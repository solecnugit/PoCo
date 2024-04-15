const WebSocket = require("ws");

// import the min approval constant which will be used to compare the count the messages
const { MIN_APPROVALS } = require("./config");

// decalre a p2p server port on which it would listen for messages
// we will pass the port through command line
const P2P_PORT = process.env.P2P_PORT || 5001;

// the neighbouring nodes socket addresses will be passed in command line
// this statemet splits them into an array
const peers = process.env.PEERS ? process.env.PEERS.split(",") : [];

const MESSAGE_TYPE = {
  transaction: "TRANSACTION",
  prepare: "PREPARE",
  pre_prepare: "PRE-PREPARE",
  commit: "COMMIT",
  round_change: "ROUND_CHANGE",
};
