import { getVersion } from "@tauri-apps/api/app";
import { checkUpdate, installUpdate } from "@tauri-apps/api/updater";
import { relaunch } from "@tauri-apps/api/process";
import { ask } from "@tauri-apps/api/dialog";
import backend from "./utils/backend";
import { setSettings } from "./state/settingsState";
import { setAppVersion } from "./state/appVersionState";
import { BootState, setBootState } from "./state/bootState";
import { batch } from "@preact/signals-react";

Promise.all([
    backend.settings.all(),
    getVersion(),
]).then(([settings, version]) => batch(() => {
    setSettings(settings);
    setAppVersion(version);
    setBootState(BootState.READY);

    if (settings.checkForUpdates) {
        checkUpdate().then(async ({ shouldUpdate, manifest }) => {
            if (shouldUpdate) {
                const yes = await ask(
                    `Version ${manifest?.version} is available. Install update now?`,
                    { title: 'Update Available', type: 'info' }
                );
                if (yes) {
                    await installUpdate();
                    await relaunch();
                }
            }
        }).catch(() => {});
    }
}));
