extern crate gamepad_rs;

use std::thread;
use std::time::Duration;

use gamepad_rs::*;

pub fn main() {
    let mut controller = ControllerContext::new().unwrap();

    //for _ in 0..10 {
    loop {
        //println!("{} devices", controller.scan_controllers());
        for i in 0..MAX_DEVICES {
            controller.update(i);
            let status = controller.state(i).status;
            if status == ControllerStatus::Connected {
                let nb_buttons;
                let nb_axis;
                {
                    let info = controller.info(i);
                    nb_buttons = info.digital_count;
                    nb_axis = info.analog_count;
                    println!(
                        "[{}] {} {} buttons {} axis",
                        i, info.name, info.digital_count, info.analog_count
                    );
                }
                {
                    let state = controller.state(i);
                    print!("\tbuttons :\n\t  A  B  X  Y  Up Do Le Ri St Bk Lt Rt LB RB\n\t");
                    for i in 0..nb_buttons {
                        print!("  {}", if state.digital_state[i] { 1 } else { 0 });
                    }
                    println!();
                    print!(
                        "\taxis :\n\t  ThumbLX  ThumbLY  LTrigger RTrigger ThumbRX  ThumbRY \n\t"
                    );
                    for i in 0..nb_axis {
                        print!("  {:1.4}", state.analog_state[i]);
                    }
                    println!();
                }
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
}
