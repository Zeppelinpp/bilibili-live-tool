#!/bin/bash

npm run tauri build && rm -rf /Applications/BiliLiveTool.app && cp -R src-tauri/target/release/bundle/macos/BiliLiveTool.app /Applications/
