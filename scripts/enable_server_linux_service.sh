#!/bin/bash

systemctl daemon-reload
systemctl enable ProjectTrackerServer.service >/dev/null 2>&1
systemctl start ProjectTrackerServer.service >/dev/null 2>&1