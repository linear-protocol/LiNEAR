const fs = require('fs');
const { init } = require('../near');

exports.command = 'set-node <address>';
exports.desc = 'Sync validators to the contract';
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

  // currentNodes is a map from nodeID to validator struct
  const currentNodes = await getValidators(signer, address);

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

  // Add
  // in case the list is too long, we cut it into chunks
  const chunks = chunkList(nodesToAdd, 10);
  for (const chunkNodes of chunks) {
    await signer.functionCall({
      contractId: address,
      methodName: 'add_validators',
      args: {
        validator_ids: chunkNodes.map(n => n.id),
        weights: chunkNodes.map(n => n.weight)
      }
    });
    console.log(`added ${chunkNodes.length} nodes`);
  }

  for (const node of nodesToUpdate) {
    await signer.functionCall({
      contractId: address,
      methodName: 'update_weight',
      args: {
        validator_id: node.id,
        weight: node.weight
      }
    }); 
    console.log(`node ${node.id} weight updated to ${node.weight}`);
  }

  for (const node of nodesToRemove) {
    await signer.functionCall({
      contractId: address,
      methodName: 'remove_validator',
      args: {
        validator_id: node.id
      }
    });
    console.log(`node ${node.id} removed`);
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

async function getValidators(signer, address) {
  let results = {};
  let offset = 0;
  const limit = 20;

  while (true) {
    const data = await signer.functionCall({
      contractId: address,
      methodName: 'get_validators',
      args: {
        offset,
        limit
      }
    });

    const rawValue = data.status.SuccessValue;
    const rawString = Buffer.from(rawValue, 'base64').toString();
    const res = JSON.parse(rawString);
    if (res.length === 0) break;

    offset += res.length;

    for (const node of res) {
      results[node.account_id] = node;
    }
  }

  return results;
}
