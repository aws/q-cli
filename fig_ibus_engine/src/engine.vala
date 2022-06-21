namespace Fig {
    public static delegate int CursorCallback (int x, int y, int w, int h);

    static CursorCallback cursor_callback;
    static bool started_by_ibus;

    class FigEngine: IBus.Engine {
        construct {
            log ("Fig", GLib.LogLevelFlags.LEVEL_INFO, "Engine constructed");
            this.set_cursor_location.connect ((x, y, w, h) => {
                cursor_callback (x, y, w, h);
            });
        }

        public FigEngine() {
            Object(
                engine_name: "Fig"
            );
        }
    }

    [CCode(cname="fig_log_warning")]
    public void log_warning(string message) {
        log ("Fig", GLib.LogLevelFlags.LEVEL_WARNING, message);
    }

    [CCode(cname="fig_engine_main")]
    public void main(bool started_by_ibus_, CursorCallback cursor_callback_) {
        started_by_ibus = started_by_ibus_;
        cursor_callback = cursor_callback_;

        IBus.init ();

        var bus = new IBus.Bus ();
        if (!bus.is_connected ()) {
            log ("Fig", GLib.LogLevelFlags.FLAG_FATAL, "Could not connect to IBus daemon");
            return;
        }

        bus.disconnected.connect (() => { IBus.quit (); });

        var factory = new IBus.Factory (bus.get_connection ());
        factory.add_engine ("fig", typeof(FigEngine));
        if (started_by_ibus) {
            log ("Fig", GLib.LogLevelFlags.LEVEL_INFO, "Managed by IBus");
            bus.request_name ("org.freedesktop.IBus.Fig", 0);
        } else {
            log ("Fig", GLib.LogLevelFlags.LEVEL_INFO, "Not managed by IBus");
            var component = new IBus.Component (
                "org.freedesktop.IBus.Fig", // name
                "Fig IBus integration component", // description
                "0.1.0", // version
                "MIT", // version
                "Fig", // author
                "https://fig.io", // homepage
                "", // command_line
                "" // textdomain
            );
            var desc = new IBus.EngineDesc (
                "fig", // name
                "Fig IBus Engine", // longname 
                "Fig IBus integration engine", // description 
                "", // language
                "MIT", // license 
                "Fig", // author
                "", // icon
                "" // layout
            );
            component.add_engine (desc);
            bus.register_component (component);
        }

        IBus.main ();
    }
}
