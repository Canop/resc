const redis = require("redis")
const Promise = require("bluebird")
Promise.promisifyAll(redis)

const inputQueue = "trt/plantA/todo-queue"
const inputSet = "trt/plantA/todo-set" // set to null if not deduplicating
const takenQueue = "trt/plantA/taken"
const outputQueue = "global/done"

function print(s){
	process.stdout.write(s)
}

async function handleTask(taskName){
	const [nature, process, product] = taskName.split("/")
	if (!product) {
		return console.warn("Invalid task name : " + taskName)
	}
	print(`Executing ${nature} for product ${product} on process ${process} `)
	for (let i=0; i<10; i++) {
		print(".")
		await Promise.delay(1000)
	}
	print(" done\n")
}

const client = redis.createClient()
console.log(`Worker listening on queue ${inputQueue}`)
;(function loop(){
	// the promisified version of brpoplpush doesn't seem to work, hence this awkward construct
	//# Take a task on input, put it on taken
	client.brpoplpush(inputQueue, takenQueue, 60, async function(err, taskName){
		if (taskName) {
			try {
				//# remove the task from the set
				if (inputSet) {
					await client.zremAsync(inputSet, taskName);
				}

				//# do the job
				await handleTask(taskName)

				//# notify the scheduler the job is done
				await client.lpushAsync(outputQueue, taskName)

				//# Remove the task from taken
				await client.lremAsync(takenQueue, 1, taskName)
			} catch (e) {
				console.warn("There was an error while pushing the task back:", e)
			}
		} else if (err) {
			console.warn("There was an error:", err)
		} else {
			console.log("I'm bored")
		}
		loop()
	})
})()



