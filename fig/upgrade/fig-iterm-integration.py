#!/usr/bin/env python3

import asyncio
import iterm2
import os

async def main(connection):
    app = await iterm2.async_get_app(connection)

    async with iterm2.FocusMonitor(connection) as monitor:
        while True:
            update = await monitor.async_get_next_update()
            window = app.current_terminal_window
            if window is None:
                continue
            current_tab = window.current_tab
            if current_tab is None:
            	continue

            if update.selected_tab_changed:
                print("The active tab is now {}, ({})".
                format(update.selected_tab_changed.tab_id, current_tab.tab_id))
                os.system('~/.fig/bin/fig bg:tab {}'.format(current_tab.tab_id))
            elif update.window_changed and update.window_changed.event == iterm2.FocusUpdateWindowChanged.Reason.TERMINAL_WINDOW_BECAME_KEY:
                print("The active tab is now {} ".
                format(current_tab.tab_id))
                os.system('~/.fig/bin/fig bg:tab {}'.format(current_tab.tab_id))

iterm2.run_forever(main)
