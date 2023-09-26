import { Routes, Route, Outlet } from "react-router-dom";
import WhatsNew from "./pages/whats-new";
import Account from "./pages/account";
import SidebarLink from "./components/sidebar/link";
import { Apps, Autocomplete, GhostText, Help, Prompt, Settings, Sparkle, User } from "./components/svg/icons";

function App() {
  return (
    <Routes>
      <Route path="/" element={<Layout />}>
        <Route index element={<WhatsNew />} />
        <Route path="help" element={<div>Help</div>} />
        <Route path="autocomplete" element={<div>Autocomplete</div>} />
        <Route path="autocomplete" element={<div>Autocomplete</div>} />
        <Route path="predict" element={<div>Predict</div>} />
        <Route path="translate" element={<div>Translate</div>} />
        <Route path="predict" element={<div>Predict</div>} />
        <Route path="translate" element={<div>Translate</div>} />
        <Route path="account" element={<Account />} />
        <Route path="integrations" element={<div>Integrations</div>} />
        <Route path="preferences" element={<div>Preferences</div>} />
      </Route>
    </Routes>
  );
}

const NAV_DATA = [
  {
    type: "link",
    name: "What's New?",
    link: "/",
  },
  {
    type: "link",
    name: "Help & Support",
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

function getIconFromName (name: string) {
  switch (name.toLowerCase()) {
    case "what's new?":
    default:
      return <Sparkle />
    case "help & support":
      return <Help />
    case "autocomplete":
      return <Autocomplete />
    case "predict":
      return <GhostText />
    case "translate":
      return <Prompt />
    case "account":
      return <User />
    case "integrations":
      return <Apps />
    case "preferences":
      return <Settings />
  }
}

function Layout() {
  return (
    <div className="flex flex-row h-screen w-full overflow-hidden">
      <nav className="w-[240px] flex-none h-full flex flex-col items-center gap-1 p-4">
        {NAV_DATA.map((item, i) =>
          item.type === "link" ? (
            <SidebarLink key={item.name} path={item.link} name={item.name} icon={getIconFromName(item.name)} count={i > 0 ? i+2 : undefined} />
          ) : (
            <div
              key={item.name}
              className="pt-4 pl-3 text-sm text-zinc-600 dark:text-zinc-400 w-full rounded-lg flex flex-row items-center font-medium"
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
