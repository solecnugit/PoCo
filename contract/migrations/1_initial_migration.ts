const Migrations = artifacts.require("Migrations");

module.exports = async function (deployer: Truffle.Deployer) {
  await deployer.deploy(Migrations);
};