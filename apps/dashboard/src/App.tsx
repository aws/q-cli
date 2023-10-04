import { Routes, Route, Outlet } from "react-router-dom";
import WhatsNew from "./pages/whats-new";
import Account from "./pages/account";
import Help from "./pages/help";
import SidebarLink from "./components/sidebar/link";
import * as Icon from "./components/svg/icons";
import Autocomplete from "./pages/autocomplete";
import Translate from "./pages/translate";
import FinishOnboarding from "./pages/onboarding";
import Predict from './pages/predict'
import Preferences from './pages/preferences'
import ModalContext from "./context/modal";
import { useState } from "react";
import Modal from "./components/modal";

function App() {
  const [modal, setModal] = useState<React.ReactNode | null>(null);
  return (
    <ModalContext.Provider value={{ modal, setModal }}>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route path="onboarding" element={<FinishOnboarding />} />
          <Route index element={<WhatsNew />} />
          <Route path="help" element={<Help />} />
          <Route path="autocomplete" element={<Autocomplete />} />
          <Route path="predict" element={<Predict />} />
          <Route path="translate" element={<Translate />} />
          <Route path="account" element={<Account />} />
          <Route path="integrations" element={<div>Integrations</div>} />
          <Route path="preferences" element={<Preferences />} />
        </Route>
      </Routes>
      {modal && <Modal>{modal}</Modal>}
    </ModalContext.Provider>
  );
}

const NAV_DATA = [
  {
    type: "link",
    name: "Finish onboarding",
    link: "/onboarding",
  },
  {
    type: "link",
    name: "What's new?",
    link: "/",
  },
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
    name: "Autocomplete",
    link: "/autocomplete",
  },
  {
    type: "link",
    name: "Predict",
    link: "/predict",
  },
  {
    type: "link",
    name: "Translate",
    link: "/translate",
  },
  {
    type: "header",
    name: "Settings",
  },
  {
    type: "link",
    name: "Account",
    link: "/account",
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

function getIconFromName(name: string) {
  switch (name.toLowerCase()) {
    case "what's new?":
    default:
      return <Icon.Sparkle />;
    case "help & support":
      return <Icon.Help />;
    case "autocomplete":
      return <Icon.Autocomplete />;
    case "predict":
      return <Icon.GhostText />;
    case "translate":
      return <Icon.Prompt />;
    case "account":
      return <Icon.User />;
    case "integrations":
      return <Icon.Apps />;
    case "preferences":
      return <Icon.Settings />;
    case "finish onboarding":
      return <Icon.Onboarding />;
  }
}

function Layout() {
  return (
    <div className="flex flex-row h-screen w-full overflow-hidden">
      <nav className="w-[240px] flex-none h-full flex flex-col items-center gap-1 p-4">
        {NAV_DATA.map((item, i) =>
          item.type === "link" ? (
            <SidebarLink
              key={item.name}
              path={item.link}
              name={item.name}
              icon={getIconFromName(item.name)}
              count={i === 1 ? 10 : undefined}
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
