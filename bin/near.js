const nearAPI = require('near-api-js');

const configs = {
  testnet: {
    networkId: "testnet",
    nodeUrl: "https://public-rpc.blockpi.io/http/near-testnet",
    walletUrl: "https://wallet.testnet.near.org",
    helperUrl: "https://helper.testnet.near.org",
    explorerUrl: "https://explorer.testnet.near.org",
  },
  mainnet: {
    networkId: "mainnet",
    nodeUrl: "https://rpc.mainnet.near.org",
    walletUrl: "https://wallet.mainnet.near.org",
    helperUrl: "https://helper.mainnet.near.org",
    explorerUrl: "https://explorer.mainnet.near.org",
  }
};

/**
 * init near object
 * @param {'testnet' | 'mainnet'} network 
 * @returns
 */
exports.init = async function (network) {
  const { keyStores } = nearAPI;
  const homedir = require("os").homedir();
  const CREDENTIALS_DIR = ".near-credentials";
  const credentialsPath = require("path").join(homedir, CREDENTIALS_DIR);
  const keyStore = new keyStores.UnencryptedFileSystemKeyStore(credentialsPath);

  const config = configs[network];
  config.keyStore = keyStore;
  return nearAPI.connect(config);
}
