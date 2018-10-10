
# Purpose

Redis lists are wonderful as task queues for distributed workers. A worker can safely and atomically take a task, even when several ones watch the same queue.

Resc is a reliable and configurable task generator for redis.

It watches one or several queues for events, which can be task completion notifications or simple "root" events, and applies rules to generate tasks.

It achieves this in a safe and monitorable way and takes care, for example, of avoiding duplicating tasks.

Resc is written in rust for safety and performance.

# Examples

This examples can be found in this repository as `demo/demo.conf.json`.

## Example 1 : regex based task generation

Here's a simple configuration file:

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

## Example 2 : Fetching some data to compute new tasks

Sometimes it might be necessary to query a web service to determine what tasks must be done.

Let's say there is a REST service returning the elements which would be logically impacted when some other one change (for example a change in a customer command might involve the recomputing of some product validity for that command).

If there's certain event on product 5ab7342600000040, you want to query

     http://my-web-service/products/5ab7342600000040/direct-childs

which responds in JSON with the list of products which should be recomputed:

	[
		{"processId":634876914,"productId":"5ab7e7dc00000040"},
		{"processId":634876914,"productId":"5ab7ebe800000040"}
	]

and for each of those products you want to generate a new task.

Then the relevant rule could be like this:

	{
		"name": "TRT propagation to childs",
		"on": {
			"done": "^trt/(?P<process_id>\\d+)/(?P<product_id>\\w{16})$"
		},
		"fetch": [{
			"url": "http://my-web-service/products/${product_id}/direct-childs",
			"returns": "child"
		}],
		"todo": {
			"task": "trt/${child.processId}/${child.productId}",
			"queue": "trt/${child.processId}/todo",
			"set": "${child.productId}/todo"
		}
	}

The `fetch` element describes the HTTP query and the namespace of the variables read in the web-service's response and used for generation of tasks, queues and sets.

In our example, we'd end with two new tasks, `"trt/634876914/5ab7e7dc00000040"` (added to queue `"trt/634876914/todo"`), and `"trt/634876914/5ab7ebe800000040"` (added to queue `"trt/634876914/todo"`).

A possible variant of this rule would be to pass the task to another queue, for another watcher will be doing the fetching, thus ensuring a slow web service doesn't block the main event queue (as every watcher is on its own thread).

# Development Status

This is a very preliminary version, without any kind of guarantee.

It's still a Work In Progress and presented here for easier peer review.

# License

*To be defined*
