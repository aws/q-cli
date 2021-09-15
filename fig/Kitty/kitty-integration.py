import os
from os.path import expanduser

def on_focus_change(boss, window, data):
    tab = boss.tab_for_window(window)

    sessionId = str(tab.id) + "/" + str(window.id)
    print(sessionId)
    if data["focused"]:
        print(sessionId)
        # send update to macOS app
        os.system("fig bg:keyboard-focus-changed net.kovidgoyal.kitty:" + sessionId)

        # logging
        logpath = expanduser("~/.fig/logs/kitty.log")
        f = open(logpath, "a+")
        f.write("Focused window: " + sessionId +"\n")
        f.close()

    # add watcher payload to all windows
    watchers = window.watchers
    for w in boss.all_windows:
        if w.id != window.id:
            w.watchers.add(watchers)



    
