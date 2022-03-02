#!/usr/bin/env node
require('yargs/yargs')(process.argv.slice(2))
  .commandDir('commands')
  .env('LI')
  .demandCommand()
  .help()
  .argv
