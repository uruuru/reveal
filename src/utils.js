const { invoke } = window.__TAURI__.core;

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
        ]
            .map(s => "\t" + s)
            .join('\n');

        let backendInfos = await invoke('debug_infos');
        alert(`${backendInfos}\nFrontend:\n${frontendInfos}`);
    } catch (e) {
        alert(`Error printing debug infos: \n\t${e}.`);
    }
}


export { isMobile, printDebug };