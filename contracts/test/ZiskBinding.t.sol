// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "forge-std/Test.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {SP1Vector} from "../src/SP1Vector.sol";
import {ISP1Vector} from "../src/interfaces/ISP1Vector.sol";
import {IZiskVerifier} from "../src/interfaces/IZiskVerifier.sol";

contract MockZiskVerifier is IZiskVerifier {
    function getRootCVadcopFinal() external pure returns (bytes32) {
        return bytes32(uint256(1));
    }

    function verifySnarkProof(bytes32 programVK, bytes32 rootC, bytes calldata, bytes calldata) external pure {
        require(programVK == bytes32(uint256(3)) && rootC == bytes32(uint256(1)));
    }
}

contract ZiskBindingTest is Test {
    SP1Vector vector;

    function setUp() public {
        SP1Vector implementation = new SP1Vector();
        vector = SP1Vector(address(new ERC1967Proxy(address(implementation), "")));
        vector.initialize(
            SP1Vector.InitParameters({
                guardian: address(this),
                height: 10,
                header: bytes32(uint256(11)),
                authoritySetId: 1,
                authoritySetHash: bytes32(uint256(2)),
                headerRangeCommitmentTreeSize: 2,
                vectorProgramVkey: bytes32(uint256(3)),
                verifier: address(new MockZiskVerifier())
            })
        );
    }

    function _output(bytes32 trustedHeader) internal pure returns (bytes memory) {
        return abi.encode(SP1Vector.ProofOutputs({
            proofType: SP1Vector.ProofType.HeaderRangeProof,
            headerRangeOutputs: abi.encode(uint32(10), trustedHeader, uint64(1), bytes32(uint256(2)), uint32(11), bytes32(uint256(12)), bytes32(uint256(13)), bytes32(uint256(14)), uint32(2)),
            rotateOutputs: new bytes(96)
        }));
    }

    function _publicValues(bytes memory output) internal pure returns (bytes memory) {
        bytes memory publics = new bytes(512);
        bytes32 digest = sha256(output);
        for (uint256 i; i < 32; ++i) {
            publics[(i / 4) * 8 + (i % 4)] = digest[i];
        }
        return publics;
    }

    function _rotateOutput() internal pure returns (bytes memory) {
        return abi.encode(SP1Vector.ProofOutputs({
            proofType: SP1Vector.ProofType.RotateProof,
            headerRangeOutputs: new bytes(288),
            rotateOutputs: abi.encode(uint64(1), bytes32(uint256(2)), bytes32(uint256(4)))
        }));
    }

    function testHeaderRangeAcceptsBoundOutput() public {
        bytes memory output = _output(bytes32(uint256(11)));
        vector.commitHeaderRange("", _publicValues(output), output);
        assertEq(vector.latestBlock(), 11);
    }

    function testHeaderRangeRejectsChangedOutput() public {
        bytes memory output = _output(bytes32(uint256(11)));
        bytes memory publics = _publicValues(output);
        output[500] = 0x01;
        vm.expectRevert();
        vector.commitHeaderRange("", publics, output);
    }

    function testHeaderRangeRejectsWrongTrustedHeader() public {
        bytes memory output = _output(bytes32(uint256(99)));
        bytes memory publics = _publicValues(output);
        vm.expectRevert(ISP1Vector.TrustedHeaderMismatch.selector);
        vector.commitHeaderRange("", publics, output);
    }

    function testRotateAcceptsBoundOutput() public {
        bytes memory output = _rotateOutput();
        vector.rotate("", _publicValues(output), output);
        assertEq(vector.authoritySetIdToHash(2), bytes32(uint256(4)));
    }

    function testRotateRejectsChangedPublicValues() public {
        bytes memory output = _rotateOutput();
        bytes memory publics = _publicValues(output);
        publics[0] = 0xff;
        vm.expectRevert(ISP1Vector.InvalidPublicValues.selector);
        vector.rotate("", publics, output);
    }
}
