// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import {stdJson} from "forge-std/StdJson.sol";
import {SP1Vector} from "../src/SP1Vector.sol";
import "forge-std/Script.sol";

// Required environment variables:
// - CONTRACT_ADDRESS_{CHAIN_ID}
// - ZISK_VECTOR_PROGRAM_VKEY

contract UpdateVkeySingleScript is Script {
    using stdJson for string;

    function setUp() public {}

    /// Updates the ZisK program verification key.
    function run() external {
        vm.startBroadcast();

        string memory contractAddressKey = string.concat("CONTRACT_ADDRESS_", vm.toString(block.chainid));
        address existingProxyAddress = vm.envAddress(contractAddressKey);

        SP1Vector sp1Vector = SP1Vector(address(existingProxyAddress));

        sp1Vector.updateVectorXProgramVkey(vm.envBytes32("ZISK_VECTOR_PROGRAM_VKEY"));

        vm.stopBroadcast();
    }
}
