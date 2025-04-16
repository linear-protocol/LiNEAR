const { readFileSync, appendFileSync } = require("fs");
const nearAPI = require('near-api-js');
const { NEAR } = require("near-units");
const { init } = require("../near");
const { networkOption } = require("./common");

exports.command = 'propose-upgrade <address>';
exports.desc = 'Propose an upgrade in DAO';
exports.builder = yargs => {
  yargs
    .positional('address', {
      describe: 'Contract address to deploy to',
      type: 'string'
    })
    .option('network', networkOption)
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
  const { address, dao, v, network } = argv;
  const code = readFileSync(argv.wasm);
  console.log(`Upgrading contract ${address}`);

  const near = await init(network);
  const account = await near.account(argv.signer);

  const blobSize = (BigInt(code.length + 32) * 10n ** 19n).toString();

  // store blob first
  const outcome = await account.signAndSendTransaction(
    {
      receiverId: dao,
      actions: [
        nearAPI.transactions.functionCall(
          'store_blob',
          code,
          100000000000000,
          blobSize
        )
      ]
    }
  );
  const hash = parseHashReturnValue(outcome);
  console.log('blob hash', hash);
  
  // save blob hash to local file
  appendFileSync(`blobhash-${network}`, `${hash}\n`);

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
    attachedDeposit: NEAR.parse('0.1')
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
