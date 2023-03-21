const { NEAR } = require("near-units");
const { init, funcCall } = require("../near");
const { networkOption } = require("./common");

exports.command = 'config-lp <address>';
exports.desc = 'Config liquidity pool';
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
    .option('expected_amount', {
      describe: 'Expected NEAR amount'
    })
    .option('max_bps', {
      describe: 'max fee basis point of 10000'
    })
    .option('min_bps', {
      describe: 'min fee basis point of 10000'
    })
    .option('treasury_bps', {
      describe: 'treasury fee basis point of 10000'
    })
    .demandOption(['signer', 'expected_amount', 'max_bps', 'min_bps', 'treasury_bps'])
    .option('dao', {
      describe: 'DAO address'
    })
}

exports.handler = async function (argv) {
  const {
    address,
    expected_amount,
    max_bps,
    min_bps,
    treasury_bps,
    dao,
  } = argv;

  const config = {
    expected_near_amount: NEAR.parse(expected_amount.toString()).toString(10),
    max_fee_bps: max_bps,
    min_fee_bps: min_bps,
    treasury_fee_bps: treasury_bps,
  };
  
  const near = await init(argv.network);
  const signer = await near.account(argv.signer);

  console.log(`setting ${address} with config`, config);

  const args = {
    config,
  };

  await funcCall(
    signer,
    dao,
    `Config liquidity pool`,
    address,
    'configure_liquidity_pool',
    args,
  );

  console.log('done');
}
