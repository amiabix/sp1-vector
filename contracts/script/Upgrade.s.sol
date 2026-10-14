// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.26;

import "forge-std/console.sol";
import {Vm} from "forge-std/Vm.sol";
import {StdAssertions} from "forge-std/StdAssertions.sol";
import {Script} from "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {SP1Vector} from "../src/SP1Vector.sol";
import {TimelockedUpgradeable} from "@succinctx/upgrades/TimelockedUpgradeable.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {BaseScript} from "./Base.s.sol";

// Required environment variables:
// - CHAINS (comma separated list of chain names)
// - CONTRACT_ADDRESS_{CHAIN_ID}
// - ZISK_VECTOR_PROGRAM_VKEY
// - ZISK_VERIFIER_ADDRESS

contract UpgradeScript is BaseScript {
    using stdJson for string;

    function setUp() public {}

    string internal constant KEY = "UpgradeScript";

    /// Upgrades the proxy and updates its ZisK verifier and program key.
    function run() external multichain(KEY) broadcaster {
        string memory contractAddressKey = string.concat("CONTRACT_ADDRESS_", vm.toString(block.chainid));
        address existingProxyAddress = vm.envAddress(contractAddressKey);

        SP1Vector sp1Vector = SP1Vector(address(existingProxyAddress));

        sp1Vector.updateFreeze(true);

        // Preserve the existing proxy and storage while replacing its proof backend.
        SP1Vector newImplementation = new SP1Vector();
        sp1Vector.upgradeTo(address(newImplementation));

        // Set the approved relayer.
        sp1Vector.setCheckRelayer(true);
        sp1Vector.setRelayerApproval(vm.envAddress("RELAYER_ADDRESS"), true);

        sp1Vector.updateVerifier(vm.envAddress("ZISK_VERIFIER_ADDRESS"));
        sp1Vector.updateVectorXProgramVkey(vm.envBytes32("ZISK_VECTOR_PROGRAM_VKEY"));
        sp1Vector.updateFreeze(false);
    }
}
