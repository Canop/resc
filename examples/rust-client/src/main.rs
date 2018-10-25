extern crate redis;

use redis::Commands;
use std::io::{self, Write};
use std::time::Duration;

// in a real program, this should be provided by configuration
const REDIS_URL: &str = "redis://127.0.0.1/";
const INPUT_QUEUE: &str = "trt/plantA/todo";
const TAKEN_QUEUE: &str = "trt/plantA/taken";
const OUTPUT_QUEUE: &str = "global/done";
const WAIT_BETWEEN_DOTS: Duration = Duration::from_secs(1);

fn handle_task(task: &str) {
    match task.split("/").collect::<Vec<&str>>().as_slice() {
        [nature, process, product] => {
            print!(
                "Executing {:?} for product {:?} on process {:?} ",
                nature, product, process
            );
            for _ in 0..10 {
                std::thread::sleep(WAIT_BETWEEN_DOTS);
                print!(".");
                io::stdout().flush().ok();
            }
            println!(" done");
        }
        _ => {
            println!("Illegal task format!");
        }
    }
}

fn main() {
    let client = redis::Client::open(REDIS_URL).unwrap();
    let con = client.get_connection().unwrap();
    println!("Worker listening on queue {:?}", INPUT_QUEUE);
    loop {
        //# Take a task on input, put it on taken
        if let Ok(task) = con.brpoplpush::<_, String>(INPUT_QUEUE, TAKEN_QUEUE, 60) {
            handle_task(&task);
            if let Err(err) = con.lpush::<_, _, ()>(OUTPUT_QUEUE, &task) {
                //# notify the scheduler the job is done
                println!("error while lpushing the task back : {:?}", err);
            }
            if let Err(err) = con.lrem::<_, _, ()>(TAKEN_QUEUE, 1, &task) {
                //# Remove the task from taken
                println!("error while cleaning the taken list : {:?}", err);
            }
        } else {
            println!("I'm bored");
        }
    }
}
