use std::process::Command;
#[macro_use] extern crate scan_fmt;
use std::error::Error;
use std::str;
use std::{thread, time};

fn nvidia_setattribute(attrbn : &str, attribval : &str) -> Result<(),String>
{
    let output = Command::new("/usr/bin/nvidia-settings")
                     .args(&["-a", &(attrbn.to_owned() + "=" + attribval)])
                     .output()
                     .expect("Failed to execute command");

    let m = str::from_utf8(&output.stdout).unwrap();

    if m.find("ERROR") != None
    {
        return Err(m.to_owned())
    }

    Ok(())
}

fn nvidia_getattribute(attrbn : &str) -> Result<u32,Box<dyn Error>>
{
    let output = Command::new("/usr/bin/nvidia-settings")
                     .args(&["-q", attrbn])
                     .output()
                     .expect("Failed to execute command");

    let m = str::from_utf8(&output.stdout).unwrap();
    let (_a,_b,c) = scan_fmt!( m,  // input string
                     "Attribute {} ({}): {}.",     // format
                     String, String, u32)?;

    Ok(c)
}

fn main()
{
    let mut last_temp = 0;
    if let Ok(v) = nvidia_getattribute("[gpu:0]/GPUCoreTemp")
    {
        last_temp = v;
        println!("[gpu:0]/GPUCoreTemp               {}", v);
    }
    if let Ok(v) = nvidia_getattribute("[gpu:0]/GPUFanControlState")
    {
        println!("[gpu:0]/GPUFanControlState         {}", v);
    }
    if let Ok(v) = nvidia_getattribute("[fan:0]/GPUTargetFanSpeed")
    {
        println!("[fan:0]/GPUTargetFanSpeed         {}", v);
    }

    if let Err(x) = nvidia_setattribute("[gpu:0]/GPUFanControlState", "1")
    {
        println!("[gpu:0]/GPUFanControlState error {}", x);
        return;
    }

    ctrlc::set_handler(move || {
        println!("Received Ctrl+C!");
        println!("Setting GPUFanControlState to 0...");
        if let Err(x) = nvidia_setattribute("[gpu:0]/GPUFanControlState", "0")
        {
            println!("[gpu:0]/GPUFanControlState error {}", x);
        }
        println!("Exiting");
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let min_speed = 30;
    let min_temp = 20;

    let max_speed = 100;
    let max_temp = 85;

    let delta_temp = max_temp - min_temp;
    let delta_speed = max_speed - min_speed;

    let two_secs = time::Duration::from_secs(2);
    
    let mut target_fan_speed : f32;

    loop
    {
        if let Ok(gpu_temp) = nvidia_getattribute("[gpu:0]/GPUCoreTemp")
        {
            if last_temp != gpu_temp
            {
                last_temp = gpu_temp;

                if gpu_temp < min_temp
                {
                    target_fan_speed = min_speed as f32;
                }
                else if gpu_temp > max_temp
                {
                    target_fan_speed = max_speed as f32;
                }
                else
                {
                    let mut delta_temp_n = (gpu_temp - min_temp) as f32;
                    target_fan_speed = delta_speed as f32;

                    delta_temp_n /= delta_temp as f32;
                    target_fan_speed = target_fan_speed * delta_temp_n + min_speed as f32;
                }

                if let Err(x) = nvidia_setattribute("[fan:0]/GPUTargetFanSpeed", &(target_fan_speed as u32).to_string())
                {
                    println!("[fan:0]/GPUTargetFanSpeed error {}", x);
                    break;
                }
                println!("[gpu:0]/GPUCoreTemp               {}  [fan:0]/GPUTargetFanSpeed {}", gpu_temp, target_fan_speed as u32);
            }
        }
        else
        {
            println!("[gpu:0]/GPUCoreTemp read error");
            return;
        }
        thread::sleep(two_secs);
    }
}
