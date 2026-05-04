#!/usr/bin/env python3
"""CDP-driven DOM automation for host-tauri WebView2 window.

Connects via Chrome DevTools Protocol, performs DOM queries,
captures state, and reports metrics. No Playwright/Selenium needed.

Usage:
  export WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=19222
  python3 scripts/tauri-cdp-automate.py --port 19222
"""
import argparse
import json
import sys
import time
import urllib.request
import asyncio
import websockets

class CDPClient:
    def __init__(self, port: int = 19222):
        self.port = port
        self.ws_url = None
        self.msg_id = 0
        self.ws = None

    def connect_http(self) -> dict:
        """Get browser version info via HTTP."""
        url = f"http://127.0.0.1:{self.port}/json/version"
        resp = urllib.request.urlopen(url)
        return json.loads(resp.read())

    def list_pages(self) -> list:
        url = f"http://127.0.0.1:{self.port}/json"
        resp = urllib.request.urlopen(url)
        return json.loads(resp.read())

    async def connect_ws(self, page_index: int = 0):
        pages = self.list_pages()
        if not pages:
            raise RuntimeError("No CDP pages available")
        target = pages[page_index]
        self.ws_url = target["webSocketDebuggerUrl"]
        self.ws = await websockets.connect(self.ws_url)
        return target

    async def send(self, method: str, params: dict = None) -> dict:
        self.msg_id += 1
        msg = {"id": self.msg_id, "method": method, "params": params or {}}
        await self.ws.send(json.dumps(msg))
        # Read until we get our response (messages may be interleaved)
        while True:
            resp = json.loads(await self.ws.recv())
            if resp.get("id") == self.msg_id:
                return resp.get("result", {})
            # Events are returned without 'id' field
            if "method" in resp:
                self._last_event = resp

    async def get_document(self) -> dict:
        """Get the root DOM node."""
        return await self.send("DOM.getDocument", {"depth": -1})

    async def query_selector(self, selector: str) -> dict:
        """Find a DOM element by CSS selector, return its node."""
        doc = await self.get_document()
        root_node_id = doc.get("root", {}).get("nodeId", 1)
        result = await self.send("DOM.querySelector", {
            "nodeId": root_node_id,
            "selector": selector
        })
        return result

    async def get_text(self, selector: str) -> str:
        """Get the text content of an element by CSS selector."""
        result = await self.query_selector(selector)
        node_id = result.get("nodeId")
        if not node_id:
            return ""
        obj = await self.send("DOM.getOuterHTML", {"nodeId": node_id})
        return obj.get("outerHTML", "")

    async def get_title(self) -> str:
        """Get the document title."""
        result = await self.send("Runtime.evaluate", {
            "expression": "document.title"
        })
        return result.get("result", {}).get("value", "")

    async def get_body_text(self) -> str:
        """Get all visible text in the document body."""
        result = await self.send("Runtime.evaluate", {
            "expression": "document.body.innerText"
        })
        return result.get("result", {}).get("value", "")

    async def screenshot(self) -> str:
        """Capture screenshot as base64 PNG."""
        result = await self.send("Page.captureScreenshot", {"format": "png"})
        return result.get("data", "")

    async def close(self):
        if self.ws:
            await self.ws.close()

async def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=19222, help="CDP port")
    parser.add_argument("--action", choices=["dom", "screenshot", "title", "full"], default="full")
    args = parser.parse_args()

    client = CDPClient(args.port)

    # Verify connection
    version = client.connect_http()
    browser = version.get("Browser", "unknown")
    print(f"Browser: {browser}", file=sys.stderr)

    await client.connect_ws(0)

    if args.action in ("title", "full"):
        title = await client.get_title()
        print(f"Title: {title}")

    if args.action in ("dom", "full"):
        text = await client.get_body_text()
        print(f"Body text ({len(text)} chars):")
        print(text[:2000])

    if args.action in ("screenshot", "full"):
        b64 = await client.screenshot()
        print(f"Screenshot: {len(b64)} base64 chars")

    await client.close()

if __name__ == "__main__":
    asyncio.run(main())
