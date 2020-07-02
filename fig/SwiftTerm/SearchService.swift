//
//  File.swift
//  
//
//  Created by Miguel de Icaza on 3/5/20.
//

import Foundation

class SearchService {
    var terminal: Terminal
    //var selection: SelectionService
    //var cache: SearchSnapshot
    
    public init (terminal: Terminal)
    {
        self.terminal = terminal
    
    }
    
    /**
     * Invalidates the current search snapshot due to content or size changes.
     * The cache should be invalidated when either the content of the buffer or the buffer dimensions change
     * because the snapshot has direct mappings to buffer line and locations.
    */

    func invalidate ()
    {
    }
}
