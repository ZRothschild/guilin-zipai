const puppeteer = require('puppeteer');

(async () => {
    const browser = await puppeteer.launch({ headless: 'new' });
    const page = await browser.newPage();

    // Capture console logs
    page.on('console', msg => {
        console.log(`[BROWSER LOG]: ${msg.text()}`);
    });

    // Navigate to client
    console.log("Navigating to http://127.0.0.1:3000...");
    await page.goto('http://127.0.0.1:3000', { waitUntil: 'networkidle0' });

    // Wait to see if WS connects
    await new Promise(r => setTimeout(r, 1000));

    // Click 'Create Room'
    console.log("Clicking Create Room...");
    const createRoomBtn = await page.$x("//button[contains(text(), '创建房间')]");
    if (createRoomBtn.length > 0) {
        await createRoomBtn[0].click();
    } else {
        console.log("Create Room button not found!");
    }

    // Wait to join room
    await new Promise(r => setTimeout(r, 1000));
    console.log(`Current URL: ${page.url()}`);

    // Click 'Invite Bot'
    console.log("Clicking Invite Bot...");
    const addBotBtn = await page.$x("//button[contains(text(), '邀请机器人')]");
    if (addBotBtn.length > 0) {
        await addBotBtn[0].click();
        await new Promise(r => setTimeout(r, 500));
    } else {
        console.log("Invite Bot button not found!");
    }

    // Click 'Start Game'
    console.log("Clicking Start Game...");
    const startGameBtn = await page.$x("//button[contains(text(), '开始游戏')]");
    if (startGameBtn.length > 0) {
        await startGameBtn[0].click();
        await new Promise(r => setTimeout(r, 1000));
    } else {
        console.log("Start Game button not found!");
    }

    // Check Game Page logs
    console.log(`Current URL: ${page.url()}`);

    await browser.close();
})();
