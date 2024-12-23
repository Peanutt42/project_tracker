#!/bin/bash

systemctl daemon-reload
systemctl stop ProjectTrackerServer.service >/dev/null 2>&1 || true
systemctl disable ProjectTrackerServer.service >/dev/null 2>&1 || true
systemctl daemon-reload