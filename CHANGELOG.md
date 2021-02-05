### next
It's now possible to declare several `make` elements in a rule.

<a name="v0.2.0"></a>
### v0.2.0 - 2020-09-03
Task deduplicating changed:
- it's per task queue (thus not preventing anymore homonyms in separate queues)
- it's optional
- it doesn't prevent a task from being queued again between processing start and end

The global `task_set` property of the configuration has thus been removed and resc issues a warning when it's present.

If you want to deduplicate a task queue, you now need to
- declare a `set` (as a pattern, like the `task` and the `queue`) in the `make` part of a rule
- have the worker remove the task from the set before executing it (or after if you don't want requeuing during processing)
