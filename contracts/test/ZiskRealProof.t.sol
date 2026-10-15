// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "forge-std/Test.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {SP1Vector} from "../src/SP1Vector.sol";
import {ZiskVerifier} from "./vendor/zisk-v1.3.2-alpha/ZiskVerifier.sol";

contract ZiskRealProofTest is Test {
    bytes32 private constant PROGRAM_VK = 0xf27b07410e8543b675c8091dfd4819b71e9f05328af9d2bfee4a30eda372a4e7;

    ZiskVerifier private verifier;

    function setUp() public {
        verifier = new ZiskVerifier();
        assertEq(verifier.VERSION(), "v1.3.2-alpha");
    }

    function _fixture(string memory name)
        internal
        view
        returns (bytes memory proof, bytes memory publics, bytes memory output)
    {
        string memory prefix = string.concat("../script/test_assets/", name);
        string memory json = vm.readFile(string.concat(prefix, "-solidity.json"));
        assertEq(vm.parseJsonBytes32(json, ".programVK"), PROGRAM_VK);
        assertEq(vm.parseJsonBytes32(json, ".rootCVadcopFinal"), verifier.getRootCVadcopFinal());
        proof = vm.parseJsonBytes(json, ".proofBytes");
        publics = vm.parseJsonBytes(json, ".publicValues");
        output = vm.readFileBinary(string.concat(prefix, "-output.bin"));
    }

    function _vector(uint32 height, bytes32 header, uint64 setId, bytes32 setHash, uint32 treeSize)
        internal
        returns (SP1Vector vector)
    {
        vector = SP1Vector(address(new ERC1967Proxy(address(new SP1Vector()), "")));
        vector.initialize(
            SP1Vector.InitParameters({
                guardian: address(this),
                height: height,
                header: header,
                authoritySetId: setId,
                authoritySetHash: setHash,
                headerRangeCommitmentTreeSize: treeSize,
                vectorProgramVkey: PROGRAM_VK,
                verifier: address(verifier)
            })
        );
    }

    function testHeaderRangeProof() public {
        (bytes memory proof, bytes memory publics, bytes memory output) =
            _fixture("header-range-turing-4007140-4007141");
        SP1Vector.ProofOutputs memory decoded = abi.decode(output, (SP1Vector.ProofOutputs));
        SP1Vector.HeaderRangeOutputs memory header =
            abi.decode(decoded.headerRangeOutputs, (SP1Vector.HeaderRangeOutputs));
        SP1Vector vector = _vector(
            header.trusted_block,
            header.trusted_header_hash,
            header.authority_set_id,
            header.authority_set_hash,
            header.merkle_tree_size
        );

        vector.commitHeaderRange(proof, publics, output);

        assertEq(vector.latestBlock(), header.target_block);
        assertEq(vector.blockHeightToHeaderHash(header.target_block), header.target_header_hash);
        bytes32 key = keccak256(abi.encode(header.trusted_block, header.target_block));
        assertEq(vector.stateRootCommitments(key), header.state_root_commitment);
        assertEq(vector.dataRootCommitments(key), header.data_root_commitment);
    }

    function testRotateProof() public {
        (bytes memory proof, bytes memory publics, bytes memory output) = _fixture("rotate-synthetic");
        SP1Vector.ProofOutputs memory decoded = abi.decode(output, (SP1Vector.ProofOutputs));
        SP1Vector.RotateOutputs memory rotate = abi.decode(decoded.rotateOutputs, (SP1Vector.RotateOutputs));
        SP1Vector vector =
            _vector(1, bytes32(uint256(1)), rotate.current_authority_set_id, rotate.current_authority_set_hash, 512);

        vector.rotate(proof, publics, output);

        assertEq(vector.authoritySetIdToHash(rotate.current_authority_set_id + 1), rotate.new_authority_set_hash);
    }

    function testRejectsWrongProgramKey() public {
        (bytes memory proof, bytes memory publics, bytes memory output) =
            _fixture("header-range-turing-4007140-4007141");
        SP1Vector.ProofOutputs memory decoded = abi.decode(output, (SP1Vector.ProofOutputs));
        SP1Vector.HeaderRangeOutputs memory header =
            abi.decode(decoded.headerRangeOutputs, (SP1Vector.HeaderRangeOutputs));
        SP1Vector vector = _vector(
            header.trusted_block,
            header.trusted_header_hash,
            header.authority_set_id,
            header.authority_set_hash,
            header.merkle_tree_size
        );
        vector.updateVectorXProgramVkey(bytes32(uint256(1)));

        vm.expectRevert(ZiskVerifier.InvalidProof.selector);
        vector.commitHeaderRange(proof, publics, output);
    }
}
