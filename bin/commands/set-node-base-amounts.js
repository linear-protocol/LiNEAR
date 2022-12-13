const fs = require('fs');
const { init } = require('../near');
const prompts = require('prompts');
const { Gas } = require('near-units');

exports.command = 'set-node-base-amounts <address>';
exports.desc = 'Set base stake amounts of nodes';
exports.builder = yargs => {
  yargs
    .positional('address', {
      describe: 'LiNEAR Contract address',
      type: 'string'
    })
    .option('network', {
      describe: 'network ID',
      default: 'testnet',
      choices: ['testnet', 'mainnet']
    })
    .demandOption(['signer', 'nodes'])
    .option('signer', {
      describe: 'signer account Id to call contract'
    })
    .option('nodes', {
      describe: 'JSON file path which has nodes list'
    })
};

exports.handler = async function (argv) {
  const address = argv.address;
  const filename = argv.nodes;
  const file = fs.readFileSync(filename);
  const nodes = JSON.parse(file.toString());

  const near = await init(argv.network);
  const signer = await near.account(argv.signer);
  const contract = await near.account(address);

  // currentNodes is a map from nodeID to validator struct
  const currentNodes = await getValidators(contract);

  const nodesToUpdateBaseStakeAmount = [];

  for (const node of nodes) {
    if (!currentNodes[node.id]) {
      console.error(`Node [${node.id}] hasn't been added to list`);
      continue;
    }

    // use yoctoNEAR instead of NEAR in config to take into account staking rewards
    if (node.base_stake_amount != null && currentNodes[node.id].base != null 
      && node.base_stake_amount.toString() !== currentNodes[node.id].base.toString()) {
      nodesToUpdateBaseStakeAmount.push(node);        
    }

    delete currentNodes[node.id];
  }

  if (nodesToUpdateBaseStakeAmount.length > 0) {
    console.log("Nodes to update base stake amount:");
    console.log(nodesToUpdateBaseStakeAmount);
  } else {
    console.log("No nodes to update");
    return ;
  }

  const res = await prompts({
    type: 'confirm',
    name: 'confirm',
    message: 'Confirm update?'
  });
  if (!res.confirm) return;

  const validators = nodesToUpdateBaseStakeAmount.map(node => node.id);
  const amounts = nodesToUpdateBaseStakeAmount.map(node => node.base);
  await signer.functionCall({
    contractId: address,
    methodName: 'update_base_stake_amounts',
    args: {
      validator_ids: validators,
      amounts: amounts
    },
    gas: Gas.parse('300 Tgas')
  });

  console.log('done.');
}

async function getValidators(contract) {
  let results = {};
  let offset = 0;
  const limit = 20;

  while (true) {
    const res = await contract.viewFunction(
      contract.accountId,
      'get_validators',
      {
        offset,
        limit
      }
    );
    if (res.length === 0) break;

    offset += res.length;

    for (const node of res) {
      results[node.account_id] = node;
    }
  }

  return results;
}
