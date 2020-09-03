package com.keods.resc.examples;

import redis.clients.jedis.Jedis;
import java.util.concurrent.TimeUnit;

/**
 * A simple worker for resc, listening for a queue, "doing" the tasks,
 *  then telling redis the job is done.
 *
 * Configuration is hardcoded here.
 */
public class SimpleWorker {

	static String host = "localhost";
	static String inputQueue = "trt/plantA/todo-queue";
	static String inputSet = "trt/plantA/todo-set"; // set to null if not using a set
	static String takenQueue = "trt/plantA/taken";
	static String outputQueue = "global/done";

	static class Task {

		public final String nature;
		public final String process;
		public final String product;

		public Task(String name){
			String[] tokens = name.split("/");
			if (tokens.length!=3) throw new IllegalArgumentException("Invalid task name");
			nature = tokens[0];
			process = tokens[1];
			product = tokens[2];
		}

		public void execute() throws InterruptedException {
			System.out.printf("Executing %s for product %s on process %s ", nature, product, process);
			for (int i=0; i<10; i++) {
				System.out.print(".");
				TimeUnit.SECONDS.sleep(1);
			}
			System.out.println(" done");
		}

	}

	/**
	 * execute the task if possible
	 */
	private static void handleTask(String taskName) {
		Task task;
		try {
			task = new Task(taskName);
		} catch (IllegalArgumentException e) {
			System.out.printf("Invalid task name : \"%s\"\n", taskName);
			return;
		}
		try {
			task.execute();
		} catch (Exception e) {
			System.out.printf("Error with task \"%s\"\n", taskName);
			e.printStackTrace();
		}
	}

	public static void main(String[] args) {
		Jedis jedis = new Jedis(host);
		System.out.println("Recovering tasks from queue " + takenQueue);
		String task;
		do {
			task = jedis.rpoplpush(takenQueue, inputQueue);
			if (task != null) System.out.println("Recovered task " + task);
			else System.out.println("No more task in " + takenQueue);
		} while (task != null);

		System.out.println("Worker listening on queue " + inputQueue);
		for (;;) {
			//# Take a task on input, put it on taken
			String taskName = jedis.brpoplpush(inputQueue, takenQueue, 60);
			if (taskName != null) {
				if (inputSet != null) {
					//# remove the task from the task set
					long n = jedis.zrem(inputSet, taskName);
					if (n != 1) {
						System.out.printf(
							"Unexpected number of elements removed from set: %d\n",
							n
						);
					}
				}
				//# do the job
				handleTask(taskName);
				//# notify the scheduler the job is done
				jedis.lpush(outputQueue, taskName);
				//# Remove the task from taken
				jedis.lrem(takenQueue, 1, taskName);
			} else {
				System.out.println("I'm bored");
			}
		}
	}

}
