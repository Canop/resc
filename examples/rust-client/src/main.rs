extern crate redis;

use redis::Commands;
use std::io::{self, Write};

// in a real program, this should be provided by configuration
const redis_url: &str = "redis://127.0.0.1/";
const input_queue: &str = "trt/plantA/todo";
const taken_queue: &str = "trt/plantA/taken";
const output_queue: &str = "global/done";

fn handle_task(task: &str) {
    println!("task {:?}", task);
    match task.split("/").collect::<Vec<&str>>().as_slice() {
        [nature, process, product] => {
            print!(
                "Executing {:?} for product {:?} on process {:?} ",
                nature, product, process
            );
            let one_second = std::time::Duration::from_secs(1);
            for _ in 0..10 {
                std::thread::sleep(one_second);
                print!(".");
                io::stdout().flush();
            }
            println!(" done");
        }
        _ => {
            println!("Illegal task format!");
        }
    }
}

fn main() {
    let client = redis::Client::open(redis_url).unwrap();
    let con = client.get_connection().unwrap();
    println!("Worker listening on queue {:?}", input_queue);
    loop {
        //# Take a task on input, put it on taken
        if let Ok(task) = con.brpoplpush::<_, String>(input_queue, taken_queue, 60) {
            handle_task(&task);
            if let Err(err) = con.lpush::<_, _, ()>(output_queue, &task) {
                //# notify the scheduler the job is done
                println!("error while lpushing the task back : {:?}", err);
            }
            if let Err(err) = con.lrem::<_, _, ()>(taken_queue, 1, &task) {
                //# Remove the task from taken
                println!("error while cleaning the taken list : {:?}", err);
            }
        } else {
            println!("I'm bored");
        }
    }
}
