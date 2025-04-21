const { writeFileSync } = require("fs");
const { NEAR, Gas } = require("near-units");
const { init } = require("../near");
const nearAPI = require('near-api-js');
const { networkOption, doubleCheck, parseHashReturnValue, getBase58CodeHash } = require("./common");

exports.command = 'store-dao-contract <dao>';
exports.desc = 'Store DAO contract code from DAO factory';
exports.builder = yargs => {
  yargs
    .positional('dao', {
      describe: 'DAO contract address to deploy to',
      type: 'string'
    })
    .option('network', networkOption)
    .option('wasm', {
      describe: 'DAO contract wasm file path',
      default: 'res/sputnikdao.wasm'
    })
    .option('signer', {
      describe: 'signer account ID to call new'
    })
    .option('hash', {
      describe: 'DAO contract code hash to store'
    })
    .demandOption(['signer', 'dao', 'hash'])
}

exports.handler = async function (argv) {
  const { dao, hash, network } = argv;
  const factory = dao.split('.').slice(1).join('.');
  console.log(`Fetch DAO contract code from factory ${factory} with code hash ${hash} ...`);

  const near = await init(network);
  const signer = await near.account(argv.signer);
  const contract = await near.account(dao);

  const code = await signer.viewFunction(factory, 'get_code', { code_hash: hash }, { parse: (code) => code });
  writeFileSync(argv.wasm, code);
  const codeHash = getBase58CodeHash(code);
  if(codeHash !== hash) {
    console.error(`Fetched code hash ${codeHash} is not the same as ${hash}`);
    return;
  }

  const deposit = (BigInt(code.length + 32) * 10n ** 19n).toString()

  console.log(`Store DAO contract code with code hash ${codeHash} to DAO ${dao}`);
  console.log(`- Code hash: ${codeHash}`);
  console.log(`- Storage cost: ${NEAR.from(deposit).toHuman()}`);
  console.log(`- DAO: ${dao}`);

  const deployedCodeHash = (await contract.state()).code_hash;
  if (codeHash === deployedCodeHash) {
    console.log(
      "Contract's code hash is the same as the wasm file. There's no need to store the same code again.",
    );
    return;
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
}
