import os
from os.path import expanduser
from kittens.tui.loop import debug
from kitty.boss import Boss
from kitty.window import Window

def on_focus_change(boss: Boss, window: Window, data) -> None:
    tab = boss.tab_for_window(window)

    sessionId = str(tab.id) + "-" + str(window.id)
    if data["focused"]:
        print(sessionId)
        # send update to macOS app
        cli = expanduser("~/.fig/bin/fig")
        os.system(cli + " keyboard-focus-changed kitty " + str(window.id))

        # logging
        logpath = expanduser("~/.fig/logs/kitty.log")
        f = open(logpath, "a+")
        f.write("Focused window: " + sessionId +"\n")
        f.close()

    # add watcher payload to all windows
    # watchers = window.watchers
    # for w in boss.all_windows:
    #     if w.id != window.id:
    #         w.watchers.add(watchers)
