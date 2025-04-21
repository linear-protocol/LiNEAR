const { readFileSync, appendFileSync, existsSync } = require("fs");
const { NEAR, Gas } = require("near-units");
const { init } = require("../near");
const nearAPI = require('near-api-js');
const { networkOption, doubleCheck, parseHashReturnValue, getBase58CodeHash } = require("./common");

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

  const codeHash = getBase58CodeHash(code);
  const codeSize = code.length;
  const deposit = (BigInt(code.length + 32) * 10n ** 19n).toString()

  const near = await init(network);
  const signer = await near.account(argv.signer);
  const contract = await near.account(address);

  console.log(`Upgrading contract ${address} with wasm file of ${codeSize} bytes to ${v}`);
  console.log(`- Code hash: ${codeHash}`);
  console.log(`- Storage cost: ${NEAR.from(deposit).toHuman()}`);
  console.log(`- DAO: ${dao}`);

  const deployedCodeHash = (await contract.state()).code_hash;
  if (codeHash === deployedCodeHash) {
    console.log(
      "Contract's code hash is the same as the wasm file. There's no need to deploy the same version again.",
    );
    return;
  }

  const blobHashFile = `blobhash-${network}`;
  if (existsSync(blobHashFile)) {
    const content = readFileSync(blobHashFile).toString();
    const records = content.split('\n');
    const record = records[records.length - 2];
    const parts = record.split(',');
    const lastHash = parts[parts.length - 1];

    // If last code hash is outdated, remove the old blob first
    if (codeHash !== lastHash) {
      // remove old blob first if exists
      const found = await signer.viewFunction(dao, 'has_blob', { hash: lastHash });
      if (!found) {
        console.error(`Old blob with ${lastHash} doesn't exist. The blob might have been removed. Continue?`);
        await doubleCheck();
      } else {
        console.log(`Remove outdated blob with hash ${lastHash}. Are you sure?`);
        await doubleCheck();
        await signer.functionCall({
          contractId: dao,
          methodName: 'remove_blob',
          args: {
            hash: lastHash,
          },
        });
        console.log(`Removed blob with hash ${lastHash}`);
      }
    }
  } else {
    console.log('Blob hash file not found');
  }

  // check if the blob already exists
  const found = await signer.viewFunction(dao, 'has_blob', { hash: codeHash });
  if (found) {
    console.error(`The blob with ${codeHash} already exists. No need to store the same blob.`);
  } else {
    // store new blob
    console.log(`Store blob with hash ${codeHash}. Are you sure?`);
    await doubleCheck();
    const outcome = await signer.signAndSendTransaction(
      {
        receiverId: dao,
        actions: [
          nearAPI.transactions.functionCall(
            'store_blob',
            code,
            Gas.parse('100 Tgas'),
            deposit
          )
        ]
      }
    );
    const hash = parseHashReturnValue(outcome);
    console.log(`Stored blob with hash ${hash}`);
  }

  // save blob hash to local file
  appendFileSync(blobHashFile, `${v},${new Date().toISOString()},${codeHash}\n`);

  const proposalArgs = {
    proposal: {
      description: `Upgrade linear contract to ${v}`,
      kind: {
        UpgradeRemote: {
          receiver_id: address,
          method_name: 'upgrade',
          hash: codeHash,
        }
      }
    }
  }
  console.log(JSON.stringify(proposalArgs, undefined, 4));
  await doubleCheck();

  await signer.functionCall({
    contractId: dao,
    methodName: 'add_proposal',
    args: proposalArgs,
    attachedDeposit: NEAR.parse('0.1')
  })

  console.log('proposed!');
}
