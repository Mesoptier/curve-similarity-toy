import { type Dispatch, type SetStateAction, useEffect, useState } from 'react';

import { IPoint, JsCurve } from '@rs_lib';
import { CURVE_COLORS } from '../curves';

function makePathDefinition(curve: JsCurve): string {
    const { points } = curve;
    if (points.length === 0) {
        return '';
    }
    return 'M' + points.map(({ x, y }) => `${x},${y}`).join(' ');
}

interface CurveSpaceViewProps {
    curves: [JsCurve, JsCurve];
    updateCurves: Dispatch<SetStateAction<JsCurve[]>>;
    highlightLeash: [number, number] | null;
}

type PreviewPoints = [IPoint | null, IPoint | null];

export function CurveSpaceView(props: CurveSpaceViewProps): JSX.Element {
    const { curves, updateCurves, highlightLeash } = props;

    const [previewPoints, setPreviewPoints] = useState<PreviewPoints>([
        null,
        null,
    ]);

    useEffect(() => {
        function handleKeyboardEvent(e: KeyboardEvent) {
            setPreviewPoints((previewPoints) => {
                const curveIdx = e.ctrlKey ? 1 : 0;
                if (previewPoints[curveIdx] === null) {
                    return [previewPoints[1], previewPoints[0]];
                }
                return previewPoints;
            });
        }

        window.addEventListener('keydown', handleKeyboardEvent);
        window.addEventListener('keyup', handleKeyboardEvent);
        return () => {
            window.removeEventListener('keydown', handleKeyboardEvent);
            window.removeEventListener('keyup', handleKeyboardEvent);
        };
    }, []);

    return (
        <svg
            width={500}
            height={500}
            onClick={(e) => {
                const curveIdx = e.ctrlKey ? 1 : 0;
                const newPoint = {
                    x: e.clientX - e.currentTarget.getBoundingClientRect().x,
                    y: e.clientY - e.currentTarget.getBoundingClientRect().y,
                };

                updateCurves((curves) => {
                    curves = [...curves];
                    curves[curveIdx] = curves[curveIdx].with_point(newPoint);
                    return curves;
                });
            }}
            onMouseMove={(e) => {
                const curveIdx = e.ctrlKey ? 1 : 0;
                const newPoint = {
                    x: e.clientX - e.currentTarget.getBoundingClientRect().x,
                    y: e.clientY - e.currentTarget.getBoundingClientRect().y,
                };

                const previewPoints: PreviewPoints = [null, null];
                previewPoints[curveIdx] = newPoint;
                setPreviewPoints(previewPoints);
            }}
            onMouseLeave={() => {
                setPreviewPoints([null, null]);
            }}
            style={{ border: '1px solid gray' }}
        >
            {curves.map((curve, curveIdx) => (
                <CurvePreview
                    key={curveIdx}
                    curve={curve}
                    previewPoint={previewPoints[curveIdx]}
                    color={CURVE_COLORS[curveIdx]}
                />
            ))}
            {highlightLeash && (
                <LeashPreview curves={curves} leash={highlightLeash} />
            )}
        </svg>
    );
}

interface CurvePreviewProps {
    curve: JsCurve;
    previewPoint: IPoint | null;
    color: string;
}

function CurvePreview(props: CurvePreviewProps): JSX.Element | null {
    const { curve, previewPoint, color } = props;

    if (curve.points.length === 0) {
        return null;
    }

    const lastPoint = curve.points[curve.points.length - 1];

    return (
        <>
            {curve.points.map(({ x, y }, pointIdx) => (
                <circle key={pointIdx} cx={x} cy={y} r={5} fill={color} />
            ))}
            <path
                d={makePathDefinition(curve)}
                stroke={color}
                strokeWidth={2}
                fill="none"
            />
            {previewPoint && (
                <g style={{ opacity: 0.5 }}>
                    <circle
                        cx={previewPoint.x}
                        cy={previewPoint.y}
                        r={5}
                        fill={color}
                    />
                    <line
                        x1={lastPoint.x}
                        y1={lastPoint.y}
                        x2={previewPoint.x}
                        y2={previewPoint.y}
                        stroke={color}
                        strokeWidth={2}
                    />
                </g>
            )}
        </>
    );
}

interface LeashPreviewProps {
    curves: [JsCurve, JsCurve];
    leash: [number, number];
}

function LeashPreview(props: LeashPreviewProps): JSX.Element {
    const { curves, leash } = props;
    const points = [curves[0].at(leash[0]), curves[1].at(leash[1])];

    return (
        <>
            {points.map((point, pointIdx) => (
                <circle
                    key={pointIdx}
                    cx={point.x}
                    cy={point.y}
                    r={3}
                    fill="green"
                />
            ))}
            <line
                x1={points[0].x}
                y1={points[0].y}
                x2={points[1].x}
                y2={points[1].y}
                stroke="green"
                strokeWidth={2}
            />
        </>
    );
}
