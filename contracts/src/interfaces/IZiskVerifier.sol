// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface IZiskVerifier {
    function getRootCVadcopFinal() external view returns (bytes32);

    function verifySnarkProof(
        bytes32 programVK,
        bytes32 rootCVadcopFinal,
        bytes calldata publicValues,
        bytes calldata proofBytes
    ) external view;
}
