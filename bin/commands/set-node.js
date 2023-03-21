const fs = require('fs');
const { init } = require('../near');
const prompts = require('prompts');
const { Gas } = require('near-units');
const { networkOption } = require("./common");

exports.command = 'set-node <address>';
exports.desc = 'Sync validators to the contract';
exports.builder = yargs => {
  yargs
    .positional('address', {
      describe: 'Contract address to deploy to',
      type: 'string'
    })
    .option('network', networkOption)
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

  const nodesToAdd = [];
  const nodesToUpdate = [];
  const nodesToRemove = [];

  for (const node of nodes) {
    if (!currentNodes[node.id]) {
      nodesToAdd.push(node);
      continue;
    }

    if (node.weight.toString() !== currentNodes[node.id].weight.toString()) {
      nodesToUpdate.push(node);
    }

    delete currentNodes[node.id];
  }

  // nodes left are to remove
  for (const nodeId of Object.keys(currentNodes)) {
    nodesToRemove.push({
      id: nodeId
    });
  }

  console.log('Nodes to add:');
  console.log(nodesToAdd);

  console.log('Nodes to update:');
  console.log(nodesToUpdate);

  console.log('Nodes to remove:');
  console.log(nodesToRemove);

  const res = await prompts({
    type: 'confirm',
    name: 'confirm',
    message: 'Confirm update?'
  });
  if (!res.confirm) return;

  // Add
  // in case the list is too long, we cut it into chunks
  const chunks = chunkList(nodesToAdd, 5);
  for (const chunkNodes of chunks) {
    await signer.functionCall({
      contractId: address,
      methodName: 'add_validators',
      args: {
        validator_ids: chunkNodes.map(n => n.id),
        weights: chunkNodes.map(n => n.weight)
      },
      gas: Gas.parse('250 Tgas')
    });
    console.log(`added ${chunkNodes.length} nodes`);
  }

  // Update
  await signer.functionCall({
    contractId: address,
    methodName: 'update_weights',
    args: {
      validator_ids: nodesToUpdate.map(n => n.id),
      weights: nodesToUpdate.map(n => n.weight)
    },
    gas: Gas.parse('300 Tgas')
  });
  console.log(`Weights updated for ${nodesToUpdate.length} nodes`);

  // Remove
  // set weight to zero instead of remove it
  for (const node of nodesToRemove) {
    await signer.functionCall({
      contractId: address,
      methodName: 'update_weight',
      args: {
        validator_id: node.id,
        weight: 0
      }
    }); 
    console.log(`node ${node.id} weight set to 0`);
  }

  console.log('done.');
}

function chunkList(items, k) {
  return items.reduce((chunks, item, index) => {
    const chunkId = Math.floor(index / k);
    chunks[chunkId] = [].concat(chunks[chunkId] || [], item);
    return chunks;
  }, []);
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
