#!/usr/bin/env bash
# The Playwright MCP server for Claude Code (`.mcp.json`). Start Claude Code in the Nix dev shell;
# this script does not start the shell itself, because `nix develop path:.` would copy the whole
# directory, `.env` with its secrets too, into the world-readable `/nix/store`.
#
# The browser: on Linux the dev shell's Chromium (PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH). On macOS
# the dev shell has none, so the Chromium of the browser tests (`just web-install`, then
# `npx playwright install chromium` in web/).
set -euo pipefail

web=$(dirname "$0")/../web

executable=${PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH:-$(cd "$web" && node -p 'require("playwright-core").chromium.executablePath()')}

exec npx -y @playwright/mcp@latest --executable-path "$executable" --headless
