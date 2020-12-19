class BiMap
  <T: Hashable> {
    var fdict = Dictionary<T,T>()
    var rdict = Dictionary<T,T>()


    
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
                fdict.removeValue(forKey: key)
                rdict.removeValue(forKey: key)
            }

        }
    }

}
