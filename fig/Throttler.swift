class Throttler {
  private var workItem: DispatchWorkItem = DispatchWorkItem(block: {})
  private var previousRun: Date = Date.distantPast
  private let queue: DispatchQueue
  private let minimumDelay: TimeInterval

  init(
    minimumDelay: TimeInterval,
    queue: DispatchQueue = DispatchQueue(label: "com.withfig.keyhandler", qos: .userInitiated)
  ) {
    self.minimumDelay = minimumDelay
    self.queue = queue
  }

  func throttle(_ block: @escaping () -> Void) {
    // Cancel any existing work item if it has not yet executed
    workItem.cancel()
    // Re-assign workItem with the new block task, resetting the previousRun time when it executes
    workItem = DispatchWorkItem { [weak self] in
      self?.previousRun = Date()
      block()
    }
    // If the time since the previous run is more than the required minimum delay
    // => execute the workItem immediately
    // else
    // => delay the workItem execution by the minimum delay time
    let delay = previousRun.timeIntervalSinceNow > minimumDelay ? 0 : minimumDelay
    queue.asyncAfter(deadline: .now() + Double(delay), execute: workItem)
  }
}

class RateLimiter {
  private var workItems: [DispatchWorkItem] = []
  private var previousRun: Date = Date.distantPast
  private let queue: DispatchQueue
  private let minimumDelay: TimeInterval

  init(
    minimumDelay: TimeInterval,
    queue: DispatchQueue = DispatchQueue(label: "com.withfig.keyhandler", qos: .userInitiated)
  ) {
    self.minimumDelay = minimumDelay
    self.queue = queue
  }

  func limit(_ block: @escaping () -> Void) {

    let workItem = DispatchWorkItem { [weak self] in
      guard self?.workItems.count ?? 0 > 0  else {
        return
      }

      self?.previousRun = Date()
      self?.workItems.removeFirst()
      block()

      if self?.workItems.count ?? 0 > 0,
         let next = self?.workItems.first {
        self?.queue.asyncAfter(deadline: .now() + Double(self?.minimumDelay ?? 0), execute: next)
      }
    }

    self.queue.sync {
      self.workItems.append(workItem)
    }

    // Only schedule first workItem explicitly, others will by handled in callback
    guard self.workItems.count == 1 else {
      return
    }

    // if the previous run happened more then `minimumDelay` ago, execute immediately
    let delay = abs(previousRun.timeIntervalSinceNow) > minimumDelay ? 0 : minimumDelay
    queue.asyncAfter(deadline: .now() + Double(delay), execute: workItem)
  }

}
