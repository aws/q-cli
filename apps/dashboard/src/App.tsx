import { Routes, Route, Outlet } from "react-router-dom";
import WhatsNew from "./pages/whats-new";
import Account from "./pages/account";
import SidebarLink from "./components/sidebar/link";

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

function Layout() {
  return (
    <div className="flex flex-row h-screen w-full overflow-hidden">
      <nav className="w-80 h-full flex flex-col items-center gap-1 p-4">
        {NAV_DATA.map((item) =>
          item.type === "link" ? (
            <SidebarLink key={item.name} path={item.link} name={item.name} />
            // <Link
            //   key={item.name}
            //   to={item.link}
            //   className={cn(
            //     "px-3 py-1.5 h-10 hover:bg-[#6E3BF1]/70 text-zinc-600 dark:text-zinc-400 hover:text-white dark:hover:text-white transition-colors w-full rounded-lg flex flex-row items-center font-light",
            //     item.link === location.pathname &&
            //       "bg-[#6E3BF1] hover:bg-[#6E3BF1]/90 text-white dark:text-white"
            //   )}
            // >
            //   {item.name}
            // </Link>
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
