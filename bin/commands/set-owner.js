const { init } = require("../near");
const prompts = require('prompts');
const { networkOption } = require("./common");

exports.command = 'set-owner <address>';
exports.desc = 'Set owner';
exports.builder = yargs => {
  yargs
    .positional('address', {
      describe: 'Contract address to deploy to',
      type: 'string'
    })
    .option('network', networkOption)
    .option('signer', {
      describe: 'signer account Id to call contract'
    })
    .option('owner', {
      describe: 'new owner ID'
    })
    .demandOption(['signer', 'owner'])
}

exports.handler = async function (argv) {
  const { address, owner } = argv;
  
  const near = await init(argv.network);
  const signer = await near.account(argv.signer);

  console.log(`Setting owner to ${owner}`);

  const res = await prompts({
    type: 'confirm',
    name: 'confirm',
    message: 'Confirm update?'
  });
  if (!res.confirm) return;

  await signer.functionCall({
    contractId: address,
    methodName: 'set_owner',
    args: {
      new_owner_id: owner
    }
  });

  console.log('done');
}
