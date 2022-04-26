const { readFileSync } = require("fs");
const nearAPI = require('near-api-js');
const { NEAR } = require("near-units");
const { init } = require("../near");

exports.command = 'propose-upgrade <address>';
exports.desc = 'Propose an upgrade in DAO';
exports.builder = yargs => {
  yargs
    .positional('address', {
      describe: 'Contract address to deploy to',
      type: 'string'
    })
    .option('network', {
      describe: 'network ID',
      default: 'testnet',
      choices: ['testnet', 'mainnet']
    })
    .option('wasm', {
      describe: 'New contract wasm file path',
      default: 'res/linear.wasm'
    })
    .option('signer', {
      describe: 'signer account ID to call new'
    })
    .option('dao', {
      describe: 'DAO account Id'
    })
    .option('v', {
      describe: 'New contract version number'
    })
    .demandOption(['signer', 'dao', 'v'])
}

exports.handler = async function (argv) {
  const { address, dao, v } = argv;
  const code = readFileSync(argv.wasm);
  console.log(`Upgrading contract ${address}`);

  const near = await init(argv.network);
  const account = await near.account(argv.signer);

  // store blob first
  const outcome = await account.signAndSendTransaction(
    {
      receiverId: dao,
      actions: [
        nearAPI.transactions.functionCall(
          'store_blob',
          code,
          100000000000000,
          "5851280000000000000000000"
        )
      ]
    }
  );
  const hash = parseHashReturnValue(outcome);
  console.log('blob hash', hash);

  const proposalArgs = {
    proposal: {
      description: `Upgrade linear contract to ${v}`,
      kind: {
        UpgradeRemote: {
          receiver_id: address,
          method_name: 'upgrade',
          hash
        }
      }
    }
  }
  console.log(JSON.stringify(proposalArgs, undefined, 4));

  await account.functionCall({
    contractId: dao,
    methodName: 'add_proposal',
    args: proposalArgs,
    attachedDeposit: NEAR.parse('1')
  })

  console.log('proposed!');
}

function parseHashReturnValue(outcome) {
  const status = outcome.status;
  const data = status.SuccessValue;
  if (!data) {
    throw new Error('bad return value');
  }

  const buff = Buffer.from(data, 'base64');
  return buff.toString('ascii').replaceAll('"', "");
}
