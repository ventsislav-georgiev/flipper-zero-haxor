#![no_main]
#![no_std]

// Required for panic handler
extern crate flipperzero_rt;

use core::ffi::{c_char, c_void};
use core::mem::{size_of, MaybeUninit};
use core::ptr;

use flipperzero_rt::{entry, manifest};
use flipperzero_sys as sys;

const RECORD_GUI: *const c_char = sys::c_string!("gui");
const RECORD_NOTIFICATION: *const c_char = sys::c_string!("notification");

manifest!(name = "Haxor");
entry!(main);

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Event {
    pub tick: u8,
    pub input: *mut sys::InputEvent,
}

pub extern "C" fn draw_callback(canvas: *mut sys::Canvas, _context: *mut c_void) {
    unsafe {
        sys::canvas_clear(canvas);
        sys::canvas_set_font(canvas, sys::Font_FontPrimary);
        sys::canvas_draw_str(canvas, 0, 10, sys::c_string!("Haxor!"));
    }
}

pub extern "C" fn input_callback(input_event: *mut sys::InputEvent, ctx: *mut c_void) {
    let event_queue = ctx as *mut sys::FuriMessageQueue;
    let event = &Event {
        tick: 0,
        input: input_event,
    } as *const Event;

    unsafe {
        sys::furi_message_queue_put(event_queue, event as *const c_void, u32::MAX);
    }
}

pub extern "C" fn timer_callback(event_queue: *mut sys::FuriMessageQueue) {
    let event = &Event {
        tick: 1,
        input: ptr::null_mut(),
    } as *const Event;

    unsafe {
        sys::furi_message_queue_put(event_queue, event as *const c_void, u32::MAX);
    }
}

fn main(_args: *mut u8) -> i32 {
    let mut msg = MaybeUninit::<Event>::uninit();

    unsafe {
        let event_queue = sys::furi_message_queue_alloc(8, size_of::<Event>() as u32);
        let view_port = sys::view_port_alloc();
        sys::view_port_draw_callback_set(view_port, Some(draw_callback), ptr::null_mut());
        sys::view_port_input_callback_set(view_port, Some(input_callback), event_queue);

        let timer = sys::furi_timer_alloc(
            Some(timer_callback),
            sys::FuriTimerType_FuriTimerTypePeriodic,
            event_queue,
        );
        sys::furi_timer_start(timer, 1500);

        let gui = sys::furi_record_open(RECORD_GUI) as *mut sys::Gui;
        sys::gui_add_view_port(gui, view_port, sys::GuiLayer_GuiLayerFullscreen);

        let notifications = sys::furi_record_open(RECORD_NOTIFICATION) as *mut sys::NotificationApp;
        sys::notification_message(notifications, &sys::sequence_display_backlight_on);

        'input_loop: loop {
            let ok =
                sys::furi_message_queue_get(event_queue, msg.as_mut_ptr() as *mut c_void, u32::MAX);
            if ok != sys::FuriStatus_FuriStatusOk {
                break;
            }

            let event = msg.assume_init();
            if event.tick == 1 {
                sys::notification_message(notifications, &sys::sequence_blink_green_100);
                continue;
            }

            match (*event.input).key {
                sys::InputKey_InputKeyBack => {
                    break 'input_loop;
                }
                sys::InputKey_InputKeyLeft => {
                    sys::notification_message(notifications, &sys::sequence_blink_red_100);
                }
                sys::InputKey_InputKeyRight => {
                    sys::notification_message(notifications, &sys::sequence_blink_blue_100);
                }
                sys::InputKey_InputKeyDown => {
                    sys::notification_message(notifications, &sys::sequence_blink_yellow_100);
                }
                sys::InputKey_InputKeyUp => {
                    sys::notification_message(notifications, &sys::sequence_blink_white_100);
                }
                _ => {}
            }
        }

        sys::furi_timer_free(timer);

        sys::furi_message_queue_free(event_queue);

        sys::view_port_enabled_set(view_port, false);
        sys::gui_remove_view_port(gui, view_port);
        sys::view_port_free(view_port);
        sys::furi_record_close(RECORD_GUI);
        sys::furi_record_close(RECORD_NOTIFICATION);
    }

    return 0;
}
