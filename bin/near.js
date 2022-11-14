const nearAPI = require('near-api-js');
const { Gas, NEAR } = require("near-units");

const configs = {
  testnet: {
    networkId: "testnet",
    nodeUrl: process.env.NODE_URL_TESTNET || "https://rpc.testnet.near.org",
    walletUrl: "https://wallet.testnet.near.org",
    helperUrl: "https://helper.testnet.near.org",
    explorerUrl: "https://explorer.testnet.near.org",
  },
  mainnet: {
    networkId: "mainnet",
    nodeUrl: process.env.NODE_URL_MAINNET || "https://rpc.mainnet.near.org",
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

async function funcCallProposal(
  signer,
  dao,
  description,
  contract,
  methodName,
  args,
  deposit,
  gas,
) {
  deposit = deposit || "0";
  gas = gas || Gas.parse('100 Tgas');

  console.log('args', args);
  args = Buffer.from(JSON.stringify(args)).toString('base64');
  console.log('encoded args', args);

  const proposal = {
    proposal: {
      description,
      kind: {
        FunctionCall: {
          receiver_id: contract,
          actions: [
            {
              method_name: methodName,
              args,
              deposit,
              gas,
            }
          ]
        }
      }
    }
  };

  return signer.functionCall({
    contractId: dao,
    methodName: 'add_proposal',
    args: proposal,
    gas: Gas.parse('200 Tgas'),
    attachedDeposit: NEAR.parse('0.1')
  });
}

exports.funcCall = async function (
  signer,
  dao,
  description,
  contract,
  methodName,
  args,
  deposit,
  gas,
) {
  if (!dao) {
    return signer.functionCall({
      contractId: contract,
      methodName,
      args,
      gas,
      attachedDeposit: deposit,
    });
  } else {
    return funcCallProposal(
      signer,
      dao,
      description,
      contract,
      methodName,
      args,
      deposit,
      gas,
    );
  }
}
