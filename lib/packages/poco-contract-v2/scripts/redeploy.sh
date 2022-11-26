#!/bin/bash

if [ ! -f "${WORKSPACE_DIR}/neardev/dev-account" ]; then
    NEAR_ENV="local" pnpm run deploy
    exit $?
fi

NEAR_MODE="env" source ${WORKSPACE_DIR}/scripts/near-env.sh

CONTRACT_ID=$(cat ${WORKSPACE_DIR}/neardev/dev-account)
echo y | pnpm exec near delete $CONTRACT_ID $NEAR_HELPER_ACCOUNT
rm -rf ${WORKSPACE_DIR}/neardev/dev-account ${WORKSPACE_DIR}/neardev/dev-account.env
NEAR_ENV="local" pnpm run deploy

export CONTRACT_ID=$CONTRACT_ID