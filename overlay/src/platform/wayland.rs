//! Wayland platform implementation using layer-shell protocol
//!
//! This provides overlay windows on Wayland compositors that support
//! the wlr-layer-shell protocol (wlroots-based compositors like Hyprland, Sway, etc.)

use std::os::fd::AsFd;

use rustix::fs::{memfd_create, MemfdFlags};
use rustix::mm::{mmap, MapFlags, ProtFlags};
use wayland_client::globals::GlobalListContents;
use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_client::protocol::wl_compositor::WlCompositor;
use wayland_client::protocol::wl_region::WlRegion;
use wayland_client::protocol::wl_registry;
use wayland_client::protocol::wl_shm::{self, Format, WlShm};
use wayland_client::protocol::wl_shm_pool::WlShmPool;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::ZwlrLayerShellV1,
    zwlr_layer_surface_v1::{self, Anchor, KeyboardInteractivity, ZwlrLayerSurfaceV1},
};

use super::{OverlayConfig, OverlayPlatform, PlatformError};

/// Wayland overlay implementation
pub struct WaylandOverlay {
    config: OverlayConfig,
    connection: Connection,
    event_queue: EventQueue<WaylandState>,
    state: WaylandState,
    qh: QueueHandle<WaylandState>,
}

/// Internal state for Wayland event handling
struct WaylandState {
    running: bool,
    configured: bool,
    width: u32,
    height: u32,

    // Wayland objects
    compositor: Option<WlCompositor>,
    surface: Option<WlSurface>,
    layer_surface: Option<ZwlrLayerSurfaceV1>,
    shm: Option<WlShm>,
    buffer: Option<WlBuffer>,

    // Pixel buffer (RGBA format for rendering, converted to ARGB for Wayland)
    pixel_data: Vec<u8>,
    shm_data: Option<ShmBuffer>,
}

struct ShmBuffer {
    ptr: *mut u8,
    size: usize,
}

// SAFETY: We only access shm_data from the main thread
unsafe impl Send for ShmBuffer {}

impl WaylandState {
    fn new(width: u32, height: u32) -> Self {
        let pixel_count = (width * height) as usize;
        Self {
            running: true,
            configured: false,
            width,
            height,
            compositor: None,
            surface: None,
            layer_surface: None,
            shm: None,
            buffer: None,
            pixel_data: vec![0u8; pixel_count * 4],
            shm_data: None,
        }
    }

    fn create_shm_buffer(&mut self, qh: &QueueHandle<WaylandState>) {
        let shm = match &self.shm {
            Some(s) => s,
            None => return,
        };

        let stride = self.width * 4;
        let size = (stride * self.height) as usize;

        // Create anonymous shared memory
        let fd = memfd_create(c"baras-overlay-buffer", MemfdFlags::CLOEXEC)
            .expect("Failed to create memfd");

        rustix::fs::ftruncate(&fd, size as u64).expect("Failed to set memfd size");

        // Memory map it
        let ptr = unsafe {
            mmap(
                std::ptr::null_mut(),
                size,
                ProtFlags::READ | ProtFlags::WRITE,
                MapFlags::SHARED,
                fd.as_fd(),
                0,
            )
            .expect("Failed to mmap")
        };

        self.shm_data = Some(ShmBuffer {
            ptr: ptr as *mut u8,
            size,
        });

        // Create wayland shm pool and buffer
        let pool = shm.create_pool(fd.as_fd(), size as i32, qh, ());
        self.buffer = Some(pool.create_buffer(
            0,
            self.width as i32,
            self.height as i32,
            stride as i32,
            Format::Argb8888,
            qh,
            (),
        ));
    }

    fn copy_pixels_to_shm(&mut self) {
        let shm = match &self.shm_data {
            Some(s) => s,
            None => return,
        };

        let shm_slice = unsafe { std::slice::from_raw_parts_mut(shm.ptr, shm.size) };

        // Convert RGBA to BGRA (Wayland ARGB8888 is BGRA in little-endian)
        for (i, chunk) in self.pixel_data.chunks(4).enumerate() {
            let offset = i * 4;
            if offset + 3 < shm_slice.len() && chunk.len() == 4 {
                shm_slice[offset] = chunk[2];     // B
                shm_slice[offset + 1] = chunk[1]; // G
                shm_slice[offset + 2] = chunk[0]; // R
                shm_slice[offset + 3] = chunk[3]; // A
            }
        }
    }

    fn commit_frame(&self) {
        if let (Some(surface), Some(buffer)) = (&self.surface, &self.buffer) {
            surface.attach(Some(buffer), 0, 0);
            surface.damage_buffer(0, 0, self.width as i32, self.height as i32);
            surface.commit();
        }
    }
}

impl OverlayPlatform for WaylandOverlay {
    fn new(config: OverlayConfig) -> Result<Self, PlatformError> {
        let connection = Connection::connect_to_env()
            .map_err(|e| PlatformError::ConnectionFailed(e.to_string()))?;

        let (globals, event_queue) =
            wayland_client::globals::registry_queue_init::<WaylandState>(&connection)
                .map_err(|e| PlatformError::ConnectionFailed(e.to_string()))?;

        let qh = event_queue.handle();
        let mut state = WaylandState::new(config.width, config.height);

        // Bind globals
        let _registry = connection.display().get_registry(&qh, ());

        let compositor: WlCompositor = globals
            .bind(&qh, 4..=6, ())
            .map_err(|_| PlatformError::UnsupportedFeature("wl_compositor".to_string()))?;

        let layer_shell: ZwlrLayerShellV1 = globals
            .bind(&qh, 1..=4, ())
            .map_err(|_| PlatformError::UnsupportedFeature("zwlr_layer_shell_v1".to_string()))?;

        let shm: WlShm = globals
            .bind(&qh, 1..=1, ())
            .map_err(|_| PlatformError::UnsupportedFeature("wl_shm".to_string()))?;

        state.shm = Some(shm);

        // Create surface
        let surface = compositor.create_surface(&qh, ());
        let layer_surface = layer_shell.get_layer_surface(
            &surface,
            None,
            wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer::Overlay,
            config.namespace.clone(),
            &qh,
            (),
        );

        // Set up click-through if requested
        if config.click_through {
            let region = compositor.create_region(&qh, ());
            surface.set_input_region(Some(&region));
        }

        // Configure layer surface
        layer_surface.set_anchor(Anchor::Top | Anchor::Left);
        layer_surface.set_margin(config.y, 0, 0, config.x);
        layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer_surface.set_size(config.width, config.height);
        surface.commit();

        state.compositor = Some(compositor);
        state.surface = Some(surface);
        state.layer_surface = Some(layer_surface);

        // Create shared memory buffer
        state.create_shm_buffer(&qh);

        Ok(Self {
            config,
            connection,
            event_queue,
            state,
            qh,
        })
    }

    fn width(&self) -> u32 {
        self.state.width
    }

    fn height(&self) -> u32 {
        self.state.height
    }

    fn set_position(&mut self, x: i32, y: i32) {
        self.config.x = x;
        self.config.y = y;
        if let Some(layer_surface) = &self.state.layer_surface {
            layer_surface.set_margin(y, 0, 0, x);
        }
        if let Some(surface) = &self.state.surface {
            surface.commit();
        }
    }

    fn set_size(&mut self, width: u32, height: u32) {
        if width == self.state.width && height == self.state.height {
            return;
        }

        self.config.width = width;
        self.config.height = height;
        self.state.width = width;
        self.state.height = height;

        // Resize pixel buffer
        let pixel_count = (width * height) as usize;
        self.state.pixel_data.resize(pixel_count * 4, 0);

        // Recreate shm buffer
        self.state.create_shm_buffer(&self.qh);

        if let Some(layer_surface) = &self.state.layer_surface {
            layer_surface.set_size(width, height);
        }
        if let Some(surface) = &self.state.surface {
            surface.commit();
        }
    }

    fn set_click_through(&mut self, enabled: bool) {
        self.config.click_through = enabled;
        if let (Some(compositor), Some(surface)) = (&self.state.compositor, &self.state.surface) {
            let region = compositor.create_region(&self.qh, ());
            if !enabled {
                // Full surface is interactive
                region.add(0, 0, self.state.width as i32, self.state.height as i32);
            }
            surface.set_input_region(Some(&region));
            surface.commit();
        }
    }

    fn pixel_buffer(&mut self) -> Option<&mut [u8]> {
        Some(&mut self.state.pixel_data)
    }

    fn commit(&mut self) {
        self.state.copy_pixels_to_shm();
        self.state.commit_frame();
    }

    fn poll_events(&mut self) -> bool {
        // Flush outgoing requests first
        if self.connection.flush().is_err() {
            return false;
        }

        // Read events from the socket (non-blocking via prepare_read)
        if let Some(guard) = self.event_queue.prepare_read() {
            // Non-blocking read
            let _ = guard.read();
        }

        // Dispatch pending events
        if self.event_queue.dispatch_pending(&mut self.state).is_err() {
            return false;
        }

        self.state.running
    }

    fn run<F>(&mut self, mut render_callback: F)
    where
        F: FnMut(&mut Self),
    {
        while self.state.running {
            // Block waiting for events
            if self.event_queue.blocking_dispatch(&mut self.state).is_err() {
                break;
            }

            if self.state.configured {
                render_callback(self);
            }
        }
    }
}

// --- Wayland Dispatch implementations ---

impl Dispatch<wl_registry::WlRegistry, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlCompositor, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlCompositor,
        _event: wayland_client::protocol::wl_compositor::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlSurface, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlSurface,
        _event: wayland_client::protocol::wl_surface::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlRegion, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlRegion,
        _event: wayland_client::protocol::wl_region::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlShm, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlShm,
        _event: wl_shm::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlShmPool, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlShmPool,
        _event: wayland_client::protocol::wl_shm_pool::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlBuffer, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlBuffer,
        _event: wayland_client::protocol::wl_buffer::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrLayerShellV1, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &ZwlrLayerShellV1,
        _event: wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrLayerSurfaceV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        proxy: &ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_layer_surface_v1::Event::Configure {
                serial,
                width,
                height,
            } => {
                proxy.ack_configure(serial);

                if width > 0 && height > 0 {
                    state.width = width;
                    state.height = height;
                }

                state.configured = true;
            }
            zwlr_layer_surface_v1::Event::Closed => {
                state.running = false;
            }
            _ => {}
        }
    }
}
