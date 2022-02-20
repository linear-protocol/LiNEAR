exports.command = 'init <address>';
exports.desc = 'Init LiNEAR contract';
exports.builder = yargs => {
  yargs
    .positional('address', {
      describe: 'Contract address',
      type: 'string'
    })
    .option('network', {
      describe: 'network ID',
      default: 'testnet',
      choices: ['testnet', 'mainnet']
    })
    .demandOption(['signer', 'owner_id'])
    .option('signer', {
      describe: 'signer account ID to call new'
    })
    .option('owner_id', {
      describe: 'owner ID'
    })
};

exports.handler = async function (yargs) {
  console.log(yargs);
}
