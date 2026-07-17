interface Settings {
    processWatcherEnabled: boolean;
    pollRate: number;
    checkForUpdates: boolean;
    launchOnStart: boolean;
    hasPromptedForLaunchOnStart: boolean;
    startMinimized: boolean;
    closeToTray: boolean;
    language: string;
    loggingEnabled: boolean;
}

export default Settings;