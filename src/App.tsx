import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { checkForUpdate, settingsGet } from "./api";
import type { UpdateInfo } from "./api";
import { useStore } from "./store";
import Layout from "./components/Layout";
import UpdateBanner from "./components/UpdateBanner";
import Workbench from "./views/Workbench";
import ProjectEditor from "./views/ProjectEditor";
import MemoryBrowser from "./views/MemoryBrowser";
import GlossaryManager from "./views/GlossaryManager";
import SettingsView from "./views/Settings";
import HelpView from "./views/Help";

export default function App() {
  const view = useStore((s) => s.view);
  const setSettings = useStore((s) => s.setSettings);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);

  const { data: settings } = useQuery({
    queryKey: ["settings"],
    queryFn: settingsGet,
  });

  useEffect(() => {
    if (settings) setSettings(settings);
  }, [settings, setSettings]);

  // Check for updates once on startup, silently ignore errors.
  useEffect(() => {
    checkForUpdate()
      .then((info) => {
        if (info.available) setUpdateInfo(info);
      })
      .catch(() => {});
  }, []);

  return (
    <div className="flex flex-col h-screen overflow-hidden">
      {updateInfo && <UpdateBanner info={updateInfo} />}
      <Layout>
        {view === "workbench" && <Workbench />}
        {view === "project" && <ProjectEditor />}
        {view === "memory" && <MemoryBrowser />}
        {view === "glossary" && <GlossaryManager />}
        {view === "settings" && <SettingsView />}
        {view === "help" && <HelpView />}
      </Layout>
    </div>
  );
}
