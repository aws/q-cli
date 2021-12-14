class BiMap
<T: Hashable> {
  var fdict = [T: T]()
  var rdict = [T: T]()

  subscript(key: T) -> T? {
    get {
      if let val = fdict[key] {
        return val
      }
      if let val = rdict[key] {
        return val
      }
      return nil
    }

    set(newVal) {
      if let val = newVal {
        fdict[key] = val
        rdict[val] = key
      } else {
        if let key2 = fdict[key] {
          fdict.removeValue(forKey: key)
          rdict.removeValue(forKey: key2)
        }

        if let key2 = rdict[key] {
          rdict.removeValue(forKey: key)
          fdict.removeValue(forKey: key2)

        }
      }
    }
  }

}
