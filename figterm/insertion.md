# Insertion Lock Project

Currently figterm relies on a lockfile at ~/.fig/insertion-lock to prevent figterm from sending events while text is being inserted. This is a problem as it adds latency to figterm when inserting text via the main app.

We want to add a way for when Fig is sent a message to insert text it will determine what is currently in the edit buffer and not send any events until the insertion is complete. The issue is that the terminal could get into a degenerate state and we need to make sure that the lock will be released when the insertion is complete.

Goals:
  - [ ] Convert the lockfile to an internal lock when an insertion event is received.
  - [ ] Release the lock when the insertion is complete.
  - [ ] Add a way to determine if the insertion is complete, and if so, release the lock.
  - [ ] Add a timeout to the lock such that if the lock is not released within a certain time, we will release the lock.

# Fig on Linux

Project Description:

To move to Linux we need to be able to determine where the cursor position is.

Milestones:

Resouces:
