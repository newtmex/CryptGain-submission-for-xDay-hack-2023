# Liquid Staking Smart Contract

## Abstract

Due to the design of CryptGain and the fact that users might have an investment strategy that can not be supported by the traditional limit set for delegation by the DSC, I have made this implementation of Liquid Staking. It is a modified version of the [mx-liquid-staking-sc](https://github.com/multiversx/mx-liquid-staking-sc) project that allows users to delegate below **1 $eGLD** per transaction.

## Introduction

The Liquid Staking Smart Contract allows users to stake their **$eGLD** in return for **cryptERD**, CryptGains version liquid staked **$eGLD**, all while retaining the standard staking rewards. It presents users' position regarding their rewards earnings and the amount they can unstake. Rewards are not compounded in order to support the philosophy that CryptGain adheres to.

## Important note

The Liquid Staking SC is designed to be called by only its owner, the Postlaunch contract. Users must interact with the Postlaunch contract to effect their delegation as it holds other functionalities that make CryptGain what it is.

## Endpoints

TODO

## Testing

TODO

## Deployment

TODO
