// #![windows_subsystem = "windows"]

use crate::gui::viewer::*;
use crate::state::*;
use crate::worker::*;
use gui::compositor::Compositor;
use windows::core::Result;
use windows::core::PCSTR;
use windows::System::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::WinRT::*;
use windows::Win32::UI::WindowsAndMessaging::*;

mod graphics;
mod gui;
pub mod pass;
mod state;
mod worker;

fn func() -> Result<()> {
    unsafe {
        RoInitialize(RO_INIT_MULTITHREADED)?;
    }
    let _dispatcher_queue = DispatcherQueueWrapper::new()?;
    let mut compositor = Compositor::new()?;

    let state = RefState::new();
    // state.set_color_cloud_mode(ColorCloudMode::Enable(ColorSpace::Hsl));

    let viewer = Viewer::new(RefState::clone(&state));
    viewer.create()?;

    let mut worker = Worker::new(RefState::clone(&state), viewer.window, &mut compositor)?;

    viewer.show();
    loop {
        let mut msg = MSG::default();
        match unsafe { GetMessageA(&mut msg, None, 0, 0) } {
            BOOL(0) | BOOL(-1) => {
                break;
            }
            _ => unsafe {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            },
        }
    }
    worker.stop();
    Ok(())
}

fn main() -> Result<()> {
    let ret: Result<()> = func();

    if let Err(e) = ret {
        unsafe {
            let msg = e.message().to_string() + "\0";
            MessageBoxA(None, PCSTR(msg.as_ptr()), PCSTR("err\0".as_ptr()), MB_OK);
        }
    }

    Ok(())
}

struct DispatcherQueueWrapper {
    #[allow(unused)]
    dispatcher_queue_controller: DispatcherQueueController,

    #[allow(unused)]
    dispatcher_queue: DispatcherQueue,
}

impl DispatcherQueueWrapper {
    fn new() -> Result<Self> {
        let dispatcher_queue_controller = unsafe {
            CreateDispatcherQueueController(DispatcherQueueOptions {
                dwSize: std::mem::size_of::<DispatcherQueueOptions>() as _,
                threadType: DQTYPE_THREAD_CURRENT,
                apartmentType: DQTAT_COM_NONE,
            })
        }?;

        let dispatcher_queue = dispatcher_queue_controller.DispatcherQueue()?;

        Ok(Self {
            dispatcher_queue_controller,
            dispatcher_queue,
        })
    }
}
