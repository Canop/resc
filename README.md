
# Purpose

Redis lists are wonderful as task queues for distributed workers. A worker can safely and atomically take a task, even when several ones watch the same queue.

Resc is a reliable and configurable task generator for redis.

It watches one or several queues for events, which can be task completion notifications or simple "root" events, and applies rules to generate tasks.

It achieves this in a safe and monitorable way and takes care, for example, of avoiding duplicating tasks.

Resc is written in rust for safety and performance.

# Example

Here's a simple configuration file (present in this repository as demo/demo.conf.json):

	{
		"watchers": [
			{
				"redis": {
					"url": "redis://127.0.0.1/"
				},
				"input_queue": "global/done",
				"taken_queue": "global/taken",
				"rules": [
					{
						"name": "TRT computation on data acquisition",
						"on": {
							"done": "^acq/(?P<process_id>\\d+)/(?P<product_id>\\d+)$"
						},
						"todo": {
							"task": "trt/${process_id}/${product_id}",
							"queue": "trt/${process_id}/todo",
							"set": "products/${product_id}/todo"
						}
					}
				]
			}
		]
	}

Resc can be launched with this configuration using

	resc demo/demo.conf.json

or (during development)

	cargo run -- demo/demo.conf.json

Resc starts a watcher, a thread, over the specified `input_queue`.

When a new event (a string in the `global/done` list) appears, it's atomically moved (using [BRPOPLPUSH](https://redis.io/commands/brpoplpush)) to the `taken/global` list and watcher's rules are executed.

Assuming the coming task is `"acq/123/456"`, then the first (and unique) rule of our example will match, according to the regular expression in `"on"/"done"`.

Several variables are dynamically generated and valued:

	process_id = 123
	product_id = 456

Those variables are used to extrapolate the task, queue and set of the todo part of the rule.

The taks `"trt/123/456"` would then be created and pushed to the `"trt/123/todo"` queue, after having checked it's not in the sorted set `"products/456/todo"`.

The task is also referenced in this sorted set with the timestamp as score.

After having executed all rules on this task, it's cleared from the `"global/taken"` queue and the watcher goes on watching the `"global/done"` queue again for other tasks.

# Development Status

This is a very preliminary version, without any kind of guarantee.

It's still a Work In Progress and presented here for peer review.
