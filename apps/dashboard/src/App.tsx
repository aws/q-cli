import { Routes, Route, Outlet, useNavigate } from "react-router-dom";
// import WhatsNew from "./pages/whats-new";
import Account from "./pages/settings/account";
import Help from "./pages/help";
import SidebarLink from "./components/sidebar/link";
import Autocomplete from "./pages/terminal/autocomplete";
import Translate from "./pages/terminal/translate";
import Onboarding from "./pages/onboarding";
import Predict from "./pages/terminal/predict";
import Preferences from "./pages/settings/preferences";
import Integrations from "./pages/settings/integrations";
import Keybindings from "./pages/settings/keybindings";
import ModalContext from "./context/modal";
import { useEffect, useRef, useState } from "react";
import Modal from "./components/modal";
import { Auth, State, Telemetry, Event } from "@withfig/api-bindings";
import InstallModal, { LoginModal } from "./components/installs/modal";
import { getIconFromName } from "./lib/icons";
import { StoreContext } from "./context/zustand";
import { createStore } from "./lib/store";
import ListenerContext from "./context/input";
import { useLocation } from "react-router-dom";

function App() {
  const navigate = useNavigate();
  const location = useLocation();
  const store = useRef(createStore()).current;
  const [listening, setListening] = useState<string | null>(null);
  const [modal, setModal] = useState<React.ReactNode | null>(null);
  const [loggedIn, setLoggedIn] = useState<boolean | null>(null);
  const [onboardingComplete, setOnboardingComplete] = useState<boolean | null>(
    null
  );

  useEffect(() => {
    try {
      Telemetry.page("", location.pathname, { ...location });
    } catch (e) {
      // ignore errors
    }
  }, [location]);

  console.log(0, "new render", { onboardingComplete, loggedIn });

  useEffect(() => {
    console.log(1, "state changed", { onboardingComplete, loggedIn });
    if (onboardingComplete === null) {
      State.get("desktop.completedOnboarding")
        .then((r) => {
          if (!r) {
            setOnboardingComplete(false);
          }
          setOnboardingComplete(r);
        })
        .catch(() => {
          setOnboardingComplete(false);
        });
    }

    if (onboardingComplete === false) {
      console.log(2, "state changed", { onboardingComplete, loggedIn });
      setModal(<InstallModal />);
      return;
    }

    if (onboardingComplete === true && loggedIn === false) {
      console.log(3, "state changed", { onboardingComplete, loggedIn });
      setModal(<LoginModal next={() => setModal(null)} />);
    }
  }, [onboardingComplete, loggedIn]);

  useEffect(() => {
    Auth.status().then((r) => setLoggedIn(r.builderId));
  }, [loggedIn]);

  useEffect(() => {
    if (loggedIn === false) {
      setModal(<LoginModal next={() => setModal(null)} />);
    }
  }, [loggedIn]);

  useEffect(() => {
    let unsubscribe: () => void;
    let isStale = false;
    Event.subscribe("dashboard.navigate", (request) => {
      if (
        typeof request === "object" &&
        request !== null &&
        "path" in request &&
        typeof request.path === "string"
      ) {
        navigate(request.path);
      } else {
        console.error("Invalid dashboard.navigate request", request);
      }

      return { unsubscribe: false };
    })?.then((result) => {
      unsubscribe = result.unsubscribe;
      if (isStale) unsubscribe();
    });
    return () => {
      if (unsubscribe) unsubscribe();
      isStale = true;
    };
  }, [navigate]);

  return (
    <StoreContext.Provider value={store}>
      <ListenerContext.Provider value={{ listening, setListening }}>
        <ModalContext.Provider value={{ modal, setModal }}>
          <Routes>
            <Route path="/" element={<Layout />}>
              <Route index element={<Onboarding />} />
              {/* TODO make What's New the default view again once it's ready... */}
              {/* <Route path="onboarding" element={<FinishOnboarding />} /> */}
              {/* <Route index element={<WhatsNew />} /> */}
              <Route path="help" element={<Help />} />
              <Route path="autocomplete" element={<Autocomplete />} />
              <Route path="predict" element={<Predict />} />
              <Route path="translate" element={<Translate />} />
              <Route path="account" element={<Account />} />
              <Route path="keybindings" element={<Keybindings />} />
              <Route path="integrations" element={<Integrations />} />
              <Route path="preferences" element={<Preferences />} />
            </Route>
          </Routes>
          {modal && <Modal>{modal}</Modal>}
        </ModalContext.Provider>
      </ListenerContext.Provider>
    </StoreContext.Provider>
  );
}

const NAV_DATA = [
  {
    type: "link",
    name: "Getting started",
    link: "/",
  },
  // {
  //   type: "link",
  //   name: "Getting started",
  //   link: "/onboarding",
  // },
  // {
  //   type: "link",
  //   name: "What's new?",
  //   link: "/",
  // },
  {
    type: "link",
    name: "Help & support",
    link: "/help",
  },
  {
    type: "header",
    name: "Terminal",
  },
  {
    type: "link",
    name: "CLI Completions",
    link: "/autocomplete",
  },
  {
    type: "link",
    name: "Translation",
    link: "/translate",
  },
  {
    type: "link",
    name: "GhostText",
    link: "/predict",
  },
  {
    type: "header",
    name: "Settings",
  },
  // {
  //   type: "link",
  //   name: "Account",
  //   link: "/account",
  // },
  {
    type: "link",
    name: "Keybindings",
    link: "/keybindings",
  },
  {
    type: "link",
    name: "Integrations",
    link: "/integrations",
  },
  {
    type: "link",
    name: "Preferences",
    link: "/preferences",
  },
] as const;

function Layout() {
  return (
    <div className="flex flex-row h-screen w-full overflow-hidden">
      <nav className="w-[240px] flex-none h-full flex flex-col items-center gap-1 p-4">
        {NAV_DATA.map((item) =>
          item.type === "link" ? (
            <SidebarLink
              key={item.name}
              path={item.link}
              name={item.name}
              icon={getIconFromName(item.name)}
              count={undefined}
            />
          ) : (
            <div
              key={item.name}
              className="pt-4 pl-3 text-sm text-zinc-600 dark:text-zinc-400 w-full rounded-lg flex flex-row items-center font-medium select-none"
            >
              {item.name}
            </div>
          )
        )}
      </nav>
      <main className="flex flex-col overflow-y-auto p-4 w-full">
        <Outlet />
      </main>
    </div>
  );
}

export default App;
