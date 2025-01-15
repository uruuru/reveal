const { invoke } = window.__TAURI__.core;
const { message } = window.__TAURI__.dialog;

function isMobile() {
    return navigator.maxTouchPoints > 0;
}

async function printDebug() {
    try {
        const frontendInfos = [
            `isMobile: ${isMobile()}`,
            `Pointer events supported: ${window.PointerEvent ? 'true' : 'false'}`,
            `Touch events supported: ${window.TouchEvent ? 'true' : 'false'}`,
            `Max touch points: ${navigator.maxTouchPoints}`,
            `User agent: ${navigator.userAgent}`
        ].join('\n');

        let backendInfos = await invoke('debug_infos');
        await message(`${backendInfos}\n… Frontend …\n${frontendInfos}`, { title: 'Debug Information', kind: 'info' });
    } catch (e) {
        await message(`Error printing debug infos: \n\t${e}.`, { kind: 'error' });
    }
}

export { isMobile, printDebug };