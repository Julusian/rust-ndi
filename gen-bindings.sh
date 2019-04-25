#!/bin/bash

bindgen lib/Processing.NDI.Lib.h -o src/sdk.rs --no-layout-tests --whitelist-function NDIlib_v3_load --whitelist-type ".*" --whitelist-var ".*"
