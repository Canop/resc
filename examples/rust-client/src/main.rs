use {
    redis::Commands,
    std::{
        io::{self, Write},
        time::Duration,
    },
};

/// emptying the taken queue should only be done when there's only
/// one worker on that queue, or on command, after a crash
const EMPTY_TAKEN_AT_LAUNCH: bool = true;

const REDIS_URL: &str = "redis://127.0.0.1/";
const INPUT_QUEUE: &str = "trt/plantA/todo-queue";
const INPUT_SET: &str = "trt/plantA/todo-set";
const TAKEN_QUEUE: &str = "trt/plantA/taken";
const OUTPUT_QUEUE: &str = "global/events";
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
    let mut con = client.get_connection().unwrap();
    if EMPTY_TAKEN_AT_LAUNCH {
        // at launch we recover the tasks remaining in the taken_queue
        // and we move them to the list of tasks to do
        loop {
            match con.rpoplpush::<_, String>(TAKEN_QUEUE, INPUT_QUEUE) {
                Ok(task) => println!("recovered task {:?}", task),
                Err(_) => {
                    println!("No more tasks to recover in queue {:?}", TAKEN_QUEUE);
                    break;
                },
            }
        }
    }
    println!("Worker listening on queue {:?}", INPUT_QUEUE);
    loop {
        //# Take a task on input, put it on taken
        if let Ok(task) = con.brpoplpush::<_, String>(INPUT_QUEUE, TAKEN_QUEUE, 60) {
            //# removing the task from the task_set so that it can be pushed again
            match con.zrem::<_, _, i32>(INPUT_SET, &task) {
                Ok(1) => {
                    println!("removed task from set");
                }
                Ok(0) => {
                    println!("no task found in set - might be a bad configuration");
                }
                Ok(n) => {
                    println!("unexpected {} tasks removed - bad configuration", n);
                }
                Err(err) => {
                    println!("error while lpushing the task back : {:?}", err);
                }
            }
            //# do the real job
            handle_task(&task);
            //# notify the scheduler the job is done
            if let Err(err) = con.lpush::<_, _, ()>(OUTPUT_QUEUE, &task) {
                println!("error while lpushing the task back : {:?}", err);
            }
            //# Remove the task from taken
            if let Err(err) = con.lrem::<_, _, ()>(TAKEN_QUEUE, 1, &task) {
                println!("error while cleaning the taken list : {:?}", err);
            }
        } else {
            println!("I'm bored");
        }
    }
}
